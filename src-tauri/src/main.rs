// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;

fn main() {
	devices::initialise_devices();

	match tauri::Builder::default().run(tauri::generate_context!()) {
		Err(e) => panic!("Error while running Tauri application: {}", e.to_string()),
		_ => {}
	}
}
