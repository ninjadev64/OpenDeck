// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod devices;
mod events;
mod plugins;
mod shared;
mod store;
mod zip_extract;

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
			frontend::get_localisations,
			frontend::instances::create_instance,
			frontend::instances::move_instance,
			frontend::instances::remove_instance,
			frontend::instances::set_state,
			frontend::instances::update_image,
			frontend::profiles::get_profiles,
			frontend::profiles::get_selected_profile,
			frontend::profiles::set_selected_profile,
			frontend::profiles::delete_profile,
			frontend::property_inspector::make_info,
			frontend::property_inspector::switch_property_inspector,
			frontend::property_inspector::open_url,
			frontend::plugins::list_plugins,
			frontend::plugins::install_plugin,
			frontend::plugins::remove_plugin,
			frontend::plugins::reload_plugin,
			frontend::settings::get_settings,
			frontend::settings::set_settings,
			frontend::settings::open_config_directory,
			frontend::settings::get_build_info
		])
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets([LogTarget::LogDir, LogTarget::Stdout])
				.level(log::LevelFilter::Debug)
				.filter(|v| {
					!matches!(
						v.target(),
						"tungstenite::handshake::server" | "tungstenite::protocol" | "tracing::span" | "zbus::object_server" | "zbus::handshake" | "zbus::connection"
					)
				})
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
		Err(error) => panic!("failed to create Tauri application: {}", error),
	};

	if std::env::args().any(|v| v == "--hide") {
		let _ = app.get_window("main").unwrap().hide();
	}

	APP_HANDLE.set(app.handle()).unwrap();

	devices::initialise_devices();
	plugins::initialise_plugins(app.handle());

	app.run(|_, _| {});
}
