// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared;
mod events;
mod plugins;
mod devices;

#[tokio::main]
async fn main() {
	let app = match tauri::Builder::default().build(tauri::generate_context!()) {
		Ok(app) => app,
		Err(e) => panic!("Failed to create Tauri application: {}", e.to_string())
	};
	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());
	
	app.run(|_, _| {});
}
