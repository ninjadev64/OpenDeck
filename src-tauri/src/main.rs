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
use tauri::{
	menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
	tray::TrayIconBuilder,
	AppHandle, Builder, Manager, WindowEvent,
};
use tauri_plugin_log::{Target, TargetKind};

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[tokio::main]
async fn main() {
	log_panics::init();

	match Builder::default()
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
		.setup(|app| {
			APP_HANDLE.set(app.handle().clone()).unwrap();

			if std::env::args().any(|v| v == "--hide") {
				let _ = app.get_webview_window("main").unwrap().hide();
			}

			let old = app.path().config_dir().unwrap().join("com.amansprojects.opendeck");
			if old.exists() {
				let _ = std::fs::rename(old, app.path().app_config_dir().unwrap());
			}

			devices::initialise_devices();
			plugins::initialise_plugins();

			let open = MenuItemBuilder::with_id("open", "Open").build(app)?;
			let hide = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
			let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
			let separator = PredefinedMenuItem::separator(app)?;
			let menu = MenuBuilder::new(app).items(&[&open, &hide, &separator, &quit]).build()?;
			let _tray = TrayIconBuilder::new()
				.menu(&menu)
				.icon(app.default_window_icon().unwrap().clone())
				.on_menu_event(move |app, event| {
					let window = app.get_webview_window("main").unwrap();
					let _ = match event.id().as_ref() {
						"open" => window.show(),
						"hide" => window.hide(),
						"quit" => {
							app.exit(0);
							Ok(())
						}
						_ => Ok(()),
					};
				})
				.build(app)?;
			Ok(())
		})
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets([Target::new(TargetKind::LogDir { file_name: None }), Target::new(TargetKind::Stdout)])
				.level(log::LevelFilter::Debug)
				.filter(|v| {
					!matches!(
						v.target(),
						"tungstenite::handshake::server" | "tungstenite::protocol" | "tracing::span" | "zbus::object_server" | "zbus::handshake" | "zbus::connection" | "os_info::imp::lsb_release"
					)
				})
				.build(),
		)
		.plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hide"])))
		.plugin(tauri_plugin_single_instance::init(|app, _, _| app.get_webview_window("main").unwrap().show().unwrap()))
		.on_window_event(|window, event| {
			if window.label() != "main" {
				return;
			}
			if let WindowEvent::CloseRequested { api, .. } = event {
				if let Ok(true) = store::get_settings().map(|store| store.value.background) {
					window.hide().unwrap();
					api.prevent_close();
				} else {
					window.app_handle().exit(0);
				}
			}
		})
		.run(tauri::generate_context!())
	{
		Ok(_) => (),
		Err(error) => panic!("failed to run Tauri application: {}", error),
	};
}
