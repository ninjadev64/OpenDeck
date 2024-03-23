use super::{send_to_plugin, GenericInstancePayload};

use crate::shared::ActionContext;
use crate::store::profiles::{get_slot, lock_mutexes, save_profile};

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
	let mut locks = lock_mutexes().await;
	let slot = match get_slot(device, "Keypad", key, &mut locks).await? {
		Some(slot) => slot,
		None => return Ok(()),
	};

	for instance in slot {
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
		.await?;
	}

	Ok(())
}

pub async fn key_up(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;

	let slot = match get_slot(device, "Keypad", key, &mut locks).await? {
		Some(slot) => slot,
		None => return Ok(()),
	};

	for instance in slot {
		instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
		let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), instance).await;

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
		.await?;
	}

	save_profile(device, &mut locks).await?;

	Ok(())
}
