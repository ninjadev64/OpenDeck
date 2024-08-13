use super::Error;

use crate::built_info;

use tauri::{command, AppHandle};
#[cfg(not(debug_assertions))]
use tauri_plugin_autostart::ManagerExt;

#[command]
pub async fn get_settings(app: AppHandle) -> Result<crate::store::Settings, Error> {
	let store = crate::store::get_settings(app).await;
	match store {
		Ok(store) => Ok(store.value),
		Err(error) => Err(error.into()),
	}
}

#[command]
pub async fn set_settings(app: AppHandle, settings: crate::store::Settings) -> Result<(), Error> {
	#[cfg(not(debug_assertions))]
	let _ = match settings.autolaunch {
		true => app.autolaunch().enable(),
		false => app.autolaunch().disable(),
	};

	crate::devices::elgato::set_brightness(settings.brightness).await;
	let mut store = match crate::store::get_settings(app).await {
		Ok(store) => store,
		Err(error) => return Err(error.into()),
	};

	store.value = settings;
	store.save()?;
	Ok(())
}

#[command]
pub fn open_config_directory(app: AppHandle) {
	#[cfg(target_os = "windows")]
	let command = "explorer";
	#[cfg(target_os = "macos")]
	let command = "open";
	#[cfg(target_os = "linux")]
	let command = "xdg-open";
	std::process::Command::new(command).arg(app.path_resolver().app_config_dir().unwrap()).spawn().unwrap();
}

#[command]
pub fn get_build_info() -> String {
	format!(
		r#"
		<details>
			<summary> OpenDeck v{} ({}) on {} </summary>
			{}
		</details>
		"#,
		built_info::PKG_VERSION,
		built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("commit hash unknown"),
		built_info::TARGET,
		built_info::DIRECT_DEPENDENCIES_STR
	)
}
