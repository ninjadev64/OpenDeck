use super::{send_to_plugin, GenericInstancePayload};

use crate::shared::ActionContext;
use crate::store::profiles::get_instance;

use serde::Serialize;

#[derive(Serialize)]
struct KeyEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: GenericInstancePayload,
}

pub async fn key_down(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let instance = match get_instance(device, key, "Keypad").await? {
		Some(instance) => instance,
		None => return Ok(()),
	};

	send_to_plugin(
		&instance.action.plugin,
		&KeyEvent {
			event: "keyDown",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(&instance),
		},
	)
	.await
}

pub async fn key_up(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let (app, mut device_stores, devices, mut profile_stores) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match profile.keys[key as usize].as_mut() {
		Some(instance) => instance,
		None => return Ok(()),
	};

	instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
	let _ = crate::events::frontend::update_state(app.as_ref().unwrap(), instance).await;

	send_to_plugin(
		&instance.action.plugin,
		&KeyEvent {
			event: "keyUp",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(instance),
		},
	)
	.await
}
