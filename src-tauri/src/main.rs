// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod events;
mod plugins;
mod shared;
mod store;

use events::frontend;

use lazy_static::lazy_static;
use tauri::{AppHandle, Builder, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri_plugin_log::LogTarget;
use tokio::sync::Mutex;

lazy_static! {
	pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

#[tokio::main]
async fn main() {
	let tray = {
		let open = CustomMenuItem::new("open".to_string(), "Open");
		let hide = CustomMenuItem::new("hide".to_string(), "Hide");
		let quit = CustomMenuItem::new("quit".to_string(), "Quit");
		let menu = SystemTrayMenu::new().add_item(open).add_item(hide).add_native_item(SystemTrayMenuItem::Separator).add_item(quit);
		SystemTray::new().with_menu(menu)
	};

	let app = match Builder::default()
		.invoke_handler(tauri::generate_handler![
			frontend::get_devices,
			frontend::get_categories,
			frontend::create_instance,
			frontend::move_instance,
			frontend::clear_slot,
			frontend::get_profiles,
			frontend::get_selected_profile,
			frontend::set_selected_profile,
			frontend::delete_profile,
			frontend::make_info,
			frontend::switch_property_inspector,
			frontend::update_image
		])
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets([LogTarget::LogDir, LogTarget::Stdout])
				.level(log::LevelFilter::Debug)
				.build(),
		)
		.system_tray(tray)
		.on_system_tray_event(|app, event| {
			if let SystemTrayEvent::MenuItemClick { id, .. } = event {
				let window = app.get_window("main").unwrap();
				let _ = match id.as_str() {
					"open" => window.show(),
					"hide" => window.hide(),
					"quit" => {
						app.exit(0);
						Ok(())
					}
					_ => Ok(()),
				};
			}
		})
		.on_window_event(|event| {
			if let WindowEvent::CloseRequested { api, .. } = event.event() {
				event.window().hide().unwrap();
				api.prevent_close();
			}
		})
		.build(tauri::generate_context!())
	{
		Ok(app) => app,
		Err(error) => panic!("Failed to create Tauri application: {}", error),
	};

	*APP_HANDLE.lock().await = Some(app.handle());

	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());

	app.run(|_, _| {});
}
