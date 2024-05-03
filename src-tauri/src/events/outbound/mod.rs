pub mod devices;
pub mod encoder;
pub mod keypad;
pub mod property_inspector;
pub mod settings;
pub mod will_appear;

use futures_util::SinkExt;
use serde::Serialize;

#[derive(Serialize)]
struct Coordinates {
	row: u8,
	column: u8,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct GenericInstancePayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	controller: String,
	state: u16,
	isInMultiAction: bool,
}

impl GenericInstancePayload {
	fn new(instance: &crate::shared::ActionInstance, multi_action: bool) -> Self {
		let coordinates = match &instance.context.controller[..] {
			"Encoder" => Coordinates {
				row: 0,
				column: instance.context.position,
			},
			_ => Coordinates {
				row: instance.context.position / 3,
				column: instance.context.position % 3,
			},
		};

		Self {
			settings: instance.settings.clone(),
			coordinates,
			controller: instance.context.controller.clone(),
			state: instance.current_state,
			isInMultiAction: multi_action,
		}
	}
}

async fn send_to_plugin(plugin: &str, data: &impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(data)?);
	let mut sockets = super::PLUGIN_SOCKETS.lock().await;

	if let Some(socket) = sockets.get_mut(plugin) {
		socket.send(message).await?;
	} else {
		let mut queues = super::PLUGIN_QUEUES.lock().await;
		if queues.contains_key(plugin) {
			queues.get_mut(plugin).unwrap().push(message);
		} else {
			queues.insert(plugin.to_owned(), vec![message]);
		}
	}

	Ok(())
}

async fn send_to_all_plugins(data: &impl Serialize) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.get().unwrap();
	let mut entries = tokio::fs::read_dir(app.path_resolver().app_config_dir().unwrap().join("plugins/")).await?;
	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = match entry.metadata().await?.is_symlink() {
			true => tokio::fs::read_link(entry.path()).await?,
			false => entry.path(),
		};
		let metadata = tokio::fs::metadata(&path).await?;
		if metadata.is_dir() {
			let _ = send_to_plugin(entry.file_name().to_str().unwrap(), data).await;
		}
	}
	Ok(())
}

#[allow(clippy::map_entry)]
async fn send_to_property_inspector(context: &crate::shared::ActionContext, data: &impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(data)?);
	let mut sockets = super::PROPERTY_INSPECTOR_SOCKETS.lock().await;

	if let Some(socket) = sockets.get_mut(&context.to_string()) {
		socket.send(message).await?;
	} else {
		let mut queues = super::PROPERTY_INSPECTOR_QUEUES.lock().await;
		if queues.contains_key(&context.to_string()) {
			queues.get_mut(&context.to_string()).unwrap().push(message);
		} else {
			queues.insert(context.to_string(), vec![message]);
		}
	}

	Ok(())
}
