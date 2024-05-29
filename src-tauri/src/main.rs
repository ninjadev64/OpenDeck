// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod events;
mod plugins;
mod shared;
mod store;

mod built_info {
	include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use events::frontend;

use once_cell::sync::OnceCell;
use tauri::{AppHandle, Builder, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tauri_plugin_log::LogTarget;

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[tokio::main]
async fn main() {
	log_panics::init();

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
			frontend::rescan_devices,
			frontend::get_categories,
			frontend::create_instance,
			frontend::move_slot,
			frontend::clear_slot,
			frontend::remove_instance,
			frontend::get_profiles,
			frontend::get_selected_profile,
			frontend::set_selected_profile,
			frontend::delete_profile,
			frontend::make_info,
			frontend::switch_property_inspector,
			frontend::open_url,
			frontend::update_image,
			frontend::set_state,
			frontend::install_plugin,
			frontend::list_plugins,
			frontend::remove_plugin,
			frontend::get_settings,
			frontend::set_settings,
			frontend::get_localisations,
			frontend::open_config_directory,
			frontend::get_build_info
		])
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets([LogTarget::LogDir, LogTarget::Stdout])
				.level(log::LevelFilter::Debug)
				.build(),
		)
		.plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hide"])))
		.plugin(tauri_plugin_single_instance::init(|app, _, _| app.get_window("main").unwrap().show().unwrap()))
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

	if std::env::args().any(|v| v == "--hide") {
		let _ = app.get_window("main").unwrap().hide();
	}

	APP_HANDLE.set(app.handle()).unwrap();

	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());

	app.run(|_, _| {});
}
