pub mod keypad;
pub mod encoder;
pub mod settings;
pub mod property_inspector;
pub mod will_appear;

use super::{SOCKETS, PROPERTY_INSPECTOR_SOCKETS};

use serde::Serialize;
use futures_util::SinkExt;

#[derive(Serialize)]
struct Coordinates {
	row: u8,
	column: u8
}

#[derive(Serialize)]
struct GenericInstancePayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	controller: String,
	state: u16
}

impl GenericInstancePayload {
	fn new(instance: &crate::shared::ActionInstance) -> Self {
		let coordinates = match &instance.context.controller[..] {
			"Encoder" => {
				Coordinates {
					row: 0,
					column: instance.context.position
				}
			},
			_ => {
				Coordinates {
					row: instance.context.position / 3,
					column: instance.context.position % 3
				}
			}
		};

		Self {
			settings: instance.settings.clone(),
			coordinates,
			controller: instance.context.controller.clone(),
			state: instance.current_state
		}
	}
}

async fn send_to_plugin(plugin: &str, data: &impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(data).unwrap());
	let mut sockets = SOCKETS.lock().await;
	if let Some(socket) = sockets.get_mut(plugin) {
		socket.send(message).await?;
	}
	Ok(())
}

async fn send_to_property_inspector(context: &crate::shared::ActionContext, data: &impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(data).unwrap());
	let mut sockets = PROPERTY_INSPECTOR_SOCKETS.lock().await;
	if let Some(socket) = sockets.get_mut(&context.to_string()) {
		socket.send(message).await?;
	}
	Ok(())
}
