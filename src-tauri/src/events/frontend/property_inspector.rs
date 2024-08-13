use super::Error;

use crate::shared::ActionContext;

use tauri::{command, AppHandle, Manager};

#[command]
pub async fn make_info(app: AppHandle, plugin: String) -> Result<crate::plugins::info_param::Info, Error> {
	let mut path = app.path_resolver().app_config_dir().unwrap();
	path.push("plugins");
	path.push(&plugin);
	path.push("manifest.json");

	let manifest = match tokio::fs::read(&path).await {
		Ok(data) => data,
		Err(error) => return Err(anyhow::Error::from(error).into()),
	};

	let manifest: crate::plugins::manifest::PluginManifest = serde_json::from_slice(&manifest)?;

	Ok(crate::plugins::info_param::make_info(plugin, manifest.version, false).await)
}

#[command]
pub async fn switch_property_inspector(old: Option<ActionContext>, new: Option<ActionContext>) {
	if let Some(context) = old {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidDisappear").await;
	}
	if let Some(context) = new {
		let _ = crate::events::outbound::property_inspector::property_inspector_did_appear(context, "propertyInspectorDidAppear").await;
	}
}

#[command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), Error> {
	if let Err(error) = tauri::api::shell::open(&app.shell_scope(), url, None) {
		return Err(anyhow::Error::from(error).into());
	}
	Ok(())
}
