use crate::shared::config_dir;

use super::Error;

use tauri::{command, AppHandle};

#[derive(serde::Serialize)]
pub struct PluginInfo {
	id: String,
	name: String,
	author: String,
	icon: String,
	version: String,
}

#[command]
pub async fn list_plugins() -> Result<Vec<PluginInfo>, Error> {
	let mut plugins = vec![];

	let mut entries = match tokio::fs::read_dir(&config_dir().join("plugins")).await {
		Ok(entries) => entries,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = match entry.metadata().await.unwrap().is_symlink() {
			true => tokio::fs::read_link(entry.path()).await.unwrap(),
			false => entry.path(),
		};
		let metadata = tokio::fs::metadata(&path).await.unwrap();
		if metadata.is_dir() {
			let Ok(manifest) = tokio::fs::read(path.join("manifest.json")).await else { continue };
			let Ok(manifest): Result<crate::plugins::manifest::PluginManifest, _> = serde_json::from_slice(&manifest) else {
				continue;
			};
			plugins.push(PluginInfo {
				id: path.file_name().unwrap().to_str().unwrap().to_owned(),
				name: manifest.name,
				author: manifest.author,
				icon: crate::shared::convert_icon(path.join(manifest.icon).to_str().unwrap().to_owned()),
				version: manifest.version,
			});
		}
	}

	Ok(plugins)
}

#[command]
pub async fn install_plugin(app: AppHandle, id: String, url: Option<String>) -> Result<(), Error> {
	let resp = match reqwest::get(url.unwrap_or(format!("https://plugins.amankhanna.me/rezipped/{id}.zip"))).await {
		Ok(resp) => resp,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};
	let bytes = match resp.bytes().await {
		Ok(bytes) => bytes,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	let _ = crate::plugins::deactivate_plugin(&app, &format!("{}.sdPlugin", id)).await;

	let config_dir = config_dir();
	let _ = tokio::fs::create_dir_all(config_dir.join("temp")).await;

	let temp = config_dir.join("temp").join(format!("{id}.sdPlugin"));
	let actual = config_dir.join("plugins").join(format!("{id}.sdPlugin"));

	let _ = tokio::fs::rename(&actual, &temp).await;
	if let Err(error) = crate::zip_extract::extract(std::io::Cursor::new(bytes), &config_dir.join("plugins")) {
		log::error!("Failed to unzip file: {}", error.to_string());
		let _ = tokio::fs::rename(&temp, &actual).await;
		let _ = crate::plugins::initialise_plugin(&actual).await;
		return Err(anyhow::Error::from(error).into());
	}
	let _ = crate::plugins::initialise_plugin(&actual).await;
	let _ = tokio::fs::remove_dir_all(temp).await;

	Ok(())
}

#[command]
pub async fn remove_plugin(app: AppHandle, id: String) -> Result<(), Error> {
	let locks = crate::store::profiles::acquire_locks().await;
	let all = locks.profile_stores.all_from_plugin(&id);
	drop(locks);

	for context in all {
		super::instances::remove_instance(context).await?;
	}

	crate::plugins::deactivate_plugin(&app, &id).await?;
	if let Err(error) = tokio::fs::remove_dir_all(config_dir().join("plugins").join(&id)).await {
		return Err(anyhow::Error::from(error).into());
	}

	let mut categories = crate::shared::CATEGORIES.write().await;
	for category in categories.values_mut() {
		category.retain(|v| v.plugin != id);
	}
	categories.retain(|_, v| !v.is_empty());

	Ok(())
}

#[command]
pub async fn reload_plugin(app: AppHandle, id: String) {
	let _ = crate::plugins::deactivate_plugin(&app, &id).await;
	let _ = crate::plugins::initialise_plugin(&config_dir().join("plugins").join(id)).await;
}
