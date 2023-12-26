mod manifest;
mod info_param;

use crate::APP_HANDLE;
use crate::shared::{Action, CATEGORIES, convert_icon};

use std::{fs, path};
use std::process::{Command, Stdio};

use tauri::AppHandle;

use tokio::net::{TcpListener, TcpStream};
use futures_util::StreamExt;

use anyhow::{Context, anyhow};
use log::{warn, error};

/// Initialise a plugin from a given directory.
async fn initialise_plugin(path: path::PathBuf) -> anyhow::Result<()> {
	let plugin_uuid = path.file_name().unwrap().to_str().unwrap();
	let manifest_path = path.join("manifest.json");

	let manifest =
		fs::read(&manifest_path)
		.with_context(|| { format!("Failed to read manifest of plugin at {}", manifest_path.display()) })?;
	let mut manifest: manifest::PluginManifest =
		serde_json::from_slice(&manifest)
		.with_context(|| { format!("Failed to parse manifest of plugin at {}", manifest_path.display()) })?;

	for action in &mut manifest.actions {
		action.plugin = plugin_uuid.to_owned();

		let action_icon_path = path.join(action.icon.clone());
		action.icon = convert_icon(action_icon_path.to_str().unwrap().to_owned());

		if !action.property_inspector.is_empty() {
			action.property_inspector = path.join(&action.property_inspector).to_string_lossy().to_string();
		} else if let Some(ref property_inspector) = manifest.property_inspector_path {
			action.property_inspector = path.join(property_inspector).to_string_lossy().to_string();
		}

		for state in &mut action.states {
			if state.image == "actionDefaultImage" {
				state.image = action.icon.clone();
			} else {
				let state_icon = path.join(state.image.clone());
				state.image = convert_icon(state_icon.to_str().unwrap().to_owned());
			}
		}
	}

	let mut categories = CATEGORIES.lock().await;
	if let Some(category) = categories.get_mut(&manifest.category) {
		for action in manifest.actions {
			category.push(action);
		}
	} else {
		let mut category: Vec<Action> = vec![];
		for action in manifest.actions {
			category.push(action);
		}
		categories.insert(manifest.category, category);
	}

	#[cfg(target_os = "windows")]
	let platform = "windows";
	#[cfg(target_os = "macos")]
	let platform = "mac";
	#[cfg(target_os = "linux")]
	let platform = "linux";

	let mut code_path = manifest.code_path;
	let mut use_browser = false;
	let mut use_wine = false;
	let mut supported = false;

	// Determine the method used to run the plugin based on its supported operating systems and the current operating system.
	for os in manifest.os {
		if os.platform == platform {
			#[cfg(target_os = "windows")]
			if manifest.code_path_windows.is_some() { code_path = manifest.code_path_windows; }
			#[cfg(target_os = "macos")]
			if manifest.code_path_macos.is_some() { code_path = manifest.code_path_macos; }
			#[cfg(target_os = "linux")]
			if manifest.code_path_linux.is_some() { code_path = manifest.code_path_linux; }

			if let Some(code_path) = &code_path {
				use_browser = code_path.ends_with(".html");
			}
			use_wine = false;

			supported = true;
			break;
		} else if os.platform == "windows" {
			use_wine = true;
			supported = true;
		}
	}

	if !supported || code_path.is_none() {
		return Err(anyhow!("Failed to load plugin with ID {}: unsupported on platform {}", plugin_uuid, platform));
	}

	let mut devices: Vec<info_param::DeviceInfo> = vec![];
	for device in crate::devices::DEVICES.lock().await.values() {
		devices.push(info_param::DeviceInfo::new(device));
	}

	let info = info_param::Info {
		application: info_param::ApplicationInfo {
			font: String::from("Rubik"),
			language: String::from("en"),
			platform: platform.to_owned(),
			platformVersion: os_info::get().version().to_string(),
			version: env!("CARGO_PKG_VERSION").to_owned()
		},
		plugin: info_param::PluginInfo {
			uuid: plugin_uuid.to_owned(),
			version: manifest.version
		},
		devicePixelRatio: 0,
		colors: info_param::ColoursInfo {
			buttonPressedBackgroundColor: String::from("#303030FF"),
			buttonPressedBorderColor: String::from("#646464FF"),
			buttonPressedTextColor: String::from("#969696FF"),
			disabledColor: String::from("#F7821B59"),
			highlightColor: String::from("#F7821BFF"),
			mouseDownColor: String::from("#CF6304FF")
		},
		devices
	};

	let code_path = code_path.unwrap();

	if use_browser {
		// Create a webview window for the plugin and call its registration function.
		let url = String::from("http://localhost:57118/") + path.join(code_path).to_str().unwrap();
		let window = tauri::WindowBuilder::new(
			APP_HANDLE.lock().await.as_ref().unwrap(),
			plugin_uuid.replace('.', "_"),
			tauri::WindowUrl::External(url.parse().unwrap())
		)
			.visible(false)
			.build()
			.with_context(|| { format!("Failed to initialise plugin with ID {}", plugin_uuid) })?;

		window.eval(&format!(
			"let a = setInterval(() => {{
				try {{
					connectElgatoStreamDeckSocket({}, \"{}\", \"{}\", {});
					clearInterval(a);
				}} catch {{}}
			}}, 10);",
			57116, plugin_uuid, "register", serde_json::to_string(&info).unwrap()
		))?;
	} else if use_wine {
		// Start Wine with the appropriate arguments.
		Command::new("wine")
			.current_dir(&path)
			.args([
				code_path,
				String::from("-port"), 57116.to_string(),
				String::from("-pluginUUID"), plugin_uuid.to_owned(),
				String::from("-registerEvent"), String::from("register"),
				String::from("-info"), serde_json::to_string(&info).unwrap()
			])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.with_context(|| { format!("Failed to initialise plugin with ID {}", plugin_uuid) })?;
	} else {
		// Run the plugin's executable natively.
		Command::new(code_path)
			.current_dir(&path)
			.args([
				String::from("-port"), 57116.to_string(),
				String::from("-pluginUUID"), plugin_uuid.to_owned(),
				String::from("-registerEvent"), String::from("register"),
				String::from("-info"), serde_json::to_string(&info).unwrap()
			])
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.with_context(|| { format!("Failed to initialise plugin with ID {}", path.file_name().unwrap().to_str().unwrap()) })?;
	}

	Ok(())
}

/// Initialise plugins from the plugins directory.
pub fn initialise_plugins(app: AppHandle) {
	tokio::spawn(init_websocket_server());
	tokio::spawn(init_browser_server());

	let plugin_dir = app.path_resolver().app_config_dir().unwrap().join("plugins/");
	let _ = fs::create_dir_all(&plugin_dir);

	let entries = match fs::read_dir(&plugin_dir) {
		Ok(p) => p,
		Err(error) => {
			error!("Failed to read plugins directory at {}: {}", plugin_dir.display(), error);
			panic!()
		}
	};

	// Iterate through all directory entries in the plugins folder and initialise them as plugins if appropriate
	for entry in entries {
		if let Ok(entry) = entry {
			if entry.metadata().unwrap().is_dir() {
				tokio::spawn(async move {
					if let Err(error) = initialise_plugin(entry.path()).await {
						warn!("Failed to initialise plugin at {}: {}\n\tCaused by: {}", entry.path().display(), error, error.root_cause());
					}
				});
			} else {
				warn!("Failed to initialise plugin at {}: is a file", entry.path().display());
			}
		} else if let Err(error) = entry {
			warn!("Failed to read entry of plugins directory: {}", error)
		}
	}
}

/// Start the WebSocket server that plugins communicate with.
async fn init_websocket_server() {
	let listener = match TcpListener::bind("localhost:57116").await {
		Ok(listener) => listener,
		Err(error) => {
			error!("Failed to bind plugin WebSocket server to socket: {}", error);
			return;
		}
	};

	while let Ok((stream, _)) = listener.accept().await {
		accept_connection(stream).await;
	}
}

/// Handle incoming data from a WebSocket connection.
async fn accept_connection(stream: TcpStream) {
	let mut socket = match tokio_tungstenite::accept_async(stream).await {
		Ok(socket) => socket,
		Err(error) => {
			warn!("Failed to complete WebSocket handshake: {}", error);
			return;
		}
	};

	let register_event = socket.next().await.unwrap().unwrap();
	crate::events::register_plugin(serde_json::from_str(&register_event.into_text().unwrap()).unwrap(), socket).await;
}

/// Start a simple webserver to serve files of plugins that run in a browser environment.
async fn init_browser_server() {
	rouille::start_server("localhost:57118", move |request| {
		rouille::Response::html(std::fs::read_to_string(request.url()).unwrap_or_default())
	});
}
