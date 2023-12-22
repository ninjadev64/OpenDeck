// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared;
mod store;
mod events;
mod plugins;
mod devices;

use events::frontend;

use tokio::sync::Mutex;
use lazy_static::lazy_static;
use tauri_plugin_log::LogTarget;

lazy_static! {
	pub static ref APP_HANDLE: Mutex<Option<tauri::AppHandle>> = Mutex::new(None);
}

#[tokio::main]
async fn main() {
	let app = match tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			frontend::get_devices,
			frontend::get_categories,
			frontend::create_instance,
			frontend::clear_slot,
			frontend::get_profiles,
			frontend::get_selected_profile,
			frontend::set_selected_profile
		])
		.plugin(tauri_plugin_log::Builder::default()
			.targets([
				LogTarget::LogDir,
				LogTarget::Stdout
			])
			.level(log::LevelFilter::Debug)
			.build()
		)
		.build(tauri::generate_context!())
	{
		Ok(app) => app,
		Err(error) => panic!("Failed to create Tauri application: {}", error)
	};

	*APP_HANDLE.lock().await = Some(app.handle());

	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());

	app.run(|_, _| {});
}
