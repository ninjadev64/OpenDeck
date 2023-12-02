// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

mod shared;
mod events;
mod plugins;
mod devices;

#[tauri::command]
fn get_devices() -> Vec<devices::DeviceInfo> {
	devices::DEVICES.lock().unwrap().to_vec()
}

#[tauri::command]
fn get_categories() -> String {
	// Here, we "duplicate" the HashMap so it isn't captured in a MutexGuard, allowing it to be serialised
	let mut hash_map: HashMap<String, &Vec<shared::Action>> = HashMap::new();
	let categories = shared::CATEGORIES.lock().unwrap();
	for key in categories.keys() {
		hash_map.insert(key.to_string(), categories.get(key).unwrap());
	}
	serde_json::to_string(&hash_map).unwrap()
}

#[tokio::main]
async fn main() {
	let app = match
		tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![get_devices, get_categories])
		.build(tauri::generate_context!())
	{
		Ok(app) => app,
		Err(error) => panic!("Failed to create Tauri application: {}", error)
	};
	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());
	
	app.run(|_, _| {});
}
