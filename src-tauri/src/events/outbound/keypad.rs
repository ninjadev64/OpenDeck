use super::{Coordinates, send_to_plugin, get_instance};

use crate::shared::{ActionContext, ActionInstance};

use serde::Serialize;

#[derive(Serialize)]
struct KeyPayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	state: u16
}

#[derive(Serialize)]
struct KeyEvent {
	action: String,
	event: String,
	context: ActionContext,
	device: String,
	payload: KeyPayload
}

fn create_payload(instance: &ActionInstance) -> KeyPayload {
	KeyPayload {
		settings: instance.settings.clone(),
		coordinates: Coordinates {
			row: instance.context.position / 3,
			column: instance.context.position % 3
		},
		state: instance.current_state
	}
}

pub async fn key_down(device: String, key: u8) -> Result<(), anyhow::Error> {
	let instance = match get_instance(&device, key, "Keypad").await? {
		Some(instance) => instance,
		None => return Ok(())
	};

	send_to_plugin(&instance.action.plugin, KeyEvent {
		action: instance.action.uuid.clone(),
		event: "keyDown".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: create_payload(&instance)
	}).await
}

pub async fn key_up(device: String, key: u8) -> Result<(), anyhow::Error> {
	let instance = match get_instance(&device, key, "Keypad").await? {
		Some(instance) => instance,
		None => return Ok(())
	};

	send_to_plugin(&instance.action.plugin, KeyEvent {
		action: instance.action.uuid.clone(),
		event: "keyUp".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: create_payload(&instance)
	}).await
}
