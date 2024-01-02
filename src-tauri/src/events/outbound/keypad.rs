use super::{Coordinates, send_to_plugin};

use crate::shared::{ActionContext, ActionInstance};
use crate::store::profiles::get_instance;

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
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match profile.keys[key as usize].as_mut() {
		Some(instance) => instance,
		None => return Ok(())
	};

	instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);

	send_to_plugin(&instance.action.plugin, KeyEvent {
		action: instance.action.uuid.clone(),
		event: "keyUp".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: create_payload(&instance)
	}).await
}
