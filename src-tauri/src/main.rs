// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod application_watcher;
mod elgato;
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

	#[cfg(target_os = "linux")]
	// SAFETY: std::env::set_var can cause race conditions in multithreaded contexts. We have not spawned any other threads at this point.
	unsafe {
		std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
		std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
	}

	let app = match Builder::default()
		.invoke_handler(tauri::generate_handler![
			frontend::get_devices,
			frontend::restart,
			frontend::get_categories,
			frontend::get_localisations,
			frontend::get_applications,
			frontend::get_application_profiles,
			frontend::set_application_profiles,
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
			frontend::settings::open_log_directory,
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

			let mut settings = store::get_settings()?;
			use std::cmp::Ordering;
			use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
			match semver::Version::parse(built_info::PKG_VERSION)?.cmp(&semver::Version::parse(&settings.value.version)?) {
				Ordering::Less => {
					app.get_webview_window("main").unwrap().close().unwrap();
					app.dialog()
						.message(format!(
							"A newer version of OpenDeck created configuration files on this device. This version is v{}; please upgrade to v{} or newer.",
							built_info::PKG_VERSION,
							settings.value.version
						))
						.title("OpenDeck upgrade required")
						.kind(MessageDialogKind::Error)
						.show(|_| APP_HANDLE.get().unwrap().exit(1));
					return Ok(());
				}
				Ordering::Greater => {
					let old_version = settings.value.version.clone();
					settings.value.version = built_info::PKG_VERSION.to_owned();
					settings.save()?;
					if old_version == "0.0.0" {
						app.dialog()
							.message(
								r#"Thanks for installing OpenDeck!
If you have any issues, please reach out on any of the support channels listed on GitHub (and make sure to star the project while you're there!).

Some minimal statistics (such as operating system and plugins installed) will be collected from the next time the app starts.
If you do not wish to support development in this way, please disable statistics in the settings.

Enjoy!"#,
							)
							.title("OpenDeck has successfully been installed")
							.kind(MessageDialogKind::Info)
							.show(|_| ());
						settings.value.statistics = false;
					}
				}
				_ => {}
			}

			use tauri_plugin_aptabase::{Builder, EventTracker, InitOptions};
			app.handle().plugin(
				Builder::new(if settings.value.statistics { "A-SH-3841489320" } else { "" })
					.with_options(InitOptions {
						host: Some("https://aptabase.amankhanna.me".to_owned()),
						flush_interval: None,
					})
					.build(),
			)?;
			let _ = app.track_event("app_started", None);

			tokio::spawn(async {
				loop {
					elgato::initialise_devices().await;
					tokio::time::sleep(std::time::Duration::from_secs(10)).await;
				}
			});
			plugins::initialise_plugins();
			application_watcher::init_application_watcher();

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

			#[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
			{
				use tauri_plugin_deep_link::DeepLinkExt;
				let _ = app.deep_link().register_all();
			}

			async fn update() -> Result<(), anyhow::Error> {
				let res = reqwest::Client::new()
					.get("https://api.github.com/repos/nekename/OpenDeck/releases/latest")
					.header("Accept", "application/vnd.github+json")
					.header("User-Agent", "OpenDeck")
					.send()
					.await?
					.json::<serde_json::Value>()
					.await?;
				let tag_name = res.get("tag_name").unwrap().as_str().unwrap();
				if semver::Version::parse(built_info::PKG_VERSION)?.cmp(&semver::Version::parse(&tag_name[1..])?) == Ordering::Less {
					let app = APP_HANDLE.get().unwrap();
					app.dialog()
						.message(format!(
							"A new version of OpenDeck, {}, is available.\nUpdate description:\n\n{}",
							tag_name,
							res.get("body").map(|v| v.as_str().unwrap()).unwrap_or("No description").trim()
						))
						.title("OpenDeck update available")
						.show(|_| ());
				}

				Ok(())
			}

			if settings.value.updatecheck {
				tokio::spawn(async {
					if let Err(error) = update().await {
						log::warn!("Failed to update application: {error}");
					}
				});
			}

			Ok(())
		})
		.plugin(
			tauri_plugin_log::Builder::default()
				.targets([Target::new(TargetKind::LogDir { file_name: None }), Target::new(TargetKind::Stdout)])
				.level(log::LevelFilter::Info)
				.level_for("opendeck", log::LevelFilter::Trace)
				.build(),
		)
		.plugin(tauri_plugin_cors_fetch::init())
		.plugin(tauri_plugin_single_instance::init(|app, _, _| app.get_webview_window("main").unwrap().show().unwrap()))
		.plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hide"])))
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_deep_link::init())
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
		.build(tauri::generate_context!())
	{
		Ok(app) => app,
		Err(error) => panic!("failed to build Tauri application: {}", error),
	};

	app.run(|app, event| {
		if let tauri::RunEvent::Exit = event {
			tokio::spawn(elgato::reset_devices());
			use tauri_plugin_aptabase::EventTracker;
			let _ = app.track_event("app_exited", None);
			app.flush_events_blocking();
		}
	});
}
