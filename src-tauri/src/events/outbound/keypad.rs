use std::time::Duration;

use super::{send_to_plugin, GenericInstancePayload};

use crate::shared::{ActionContext, Context};
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
	let profile = locks.device_stores.get_device_store(device, crate::APP_HANDLE.get().unwrap())?.value.selected_profile.clone();
	let context = Context {
		device: device.to_owned(),
		profile,
		controller: "Keypad".to_owned(),
		position: key,
	};
	let slot = get_slot(&context, &mut locks).await?;

	if slot.len() == 1 {
		let instance = &slot[0];
		send_to_plugin(
			&instance.action.plugin,
			&KeyEvent {
				event: "keyDown",
				action: instance.action.uuid.clone(),
				context: instance.context.clone(),
				device: instance.context.device.clone(),
				payload: GenericInstancePayload::new(instance, false),
			},
		)
		.await?;
	} else {
		for instance in slot {
			send_to_plugin(
				&instance.action.plugin,
				&KeyEvent {
					event: "keyDown",
					action: instance.action.uuid.clone(),
					context: instance.context.clone(),
					device: instance.context.device.clone(),
					payload: GenericInstancePayload::new(instance, true),
				},
			)
			.await?;

			tokio::time::sleep(Duration::from_millis(100)).await;

			instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);
			send_to_plugin(
				&instance.action.plugin,
				&KeyEvent {
					event: "keyUp",
					action: instance.action.uuid.clone(),
					context: instance.context.clone(),
					device: instance.context.device.clone(),
					payload: GenericInstancePayload::new(instance, true),
				},
			)
			.await?;

			tokio::time::sleep(Duration::from_millis(100)).await;
		}

		save_profile(device, &mut locks).await?;
		let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), context, &mut locks).await;
	}

	Ok(())
}

pub async fn key_up(device: &str, key: u8) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;
	let profile = locks.device_stores.get_device_store(device, crate::APP_HANDLE.get().unwrap())?.value.selected_profile.clone();
	let context = Context {
		device: device.to_owned(),
		profile,
		controller: "Keypad".to_owned(),
		position: key,
	};

	let slot = get_slot(&context, &mut locks).await?;
	if slot.len() != 1 {
		return Ok(());
	}
	let instance = &mut slot[0];

	instance.current_state = (instance.current_state + 1) % (instance.states.len() as u16);

	send_to_plugin(
		&instance.action.plugin,
		&KeyEvent {
			event: "keyUp",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(instance, false),
		},
	)
	.await?;

	save_profile(device, &mut locks).await?;
	let _ = crate::events::frontend::update_state(crate::APP_HANDLE.get().unwrap(), context, &mut locks).await;

	Ok(())
}
