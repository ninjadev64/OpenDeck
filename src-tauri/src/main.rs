// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared;
mod store;
mod events;
mod plugins;
mod devices;

use events::frontend;

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
		.build(tauri::generate_context!())
	{
		Ok(app) => app,
		Err(error) => panic!("Failed to create Tauri application: {}", error)
	};

	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());

	app.run(|_, _| {});
}
