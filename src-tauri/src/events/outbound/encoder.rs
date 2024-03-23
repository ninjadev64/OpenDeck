use super::{send_to_plugin, Coordinates};

use crate::shared::ActionContext;
use crate::store::profiles::{get_instance, lock_mutexes};

use serde::Serialize;

#[derive(Serialize)]
struct DialRotatePayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	ticks: i16,
	pressed: bool,
}

#[derive(Serialize)]
struct DialRotateEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: DialRotatePayload,
}

#[derive(Serialize)]
struct DialPressPayload {
	controller: &'static str,
	settings: serde_json::Value,
	coordinates: Coordinates,
}

#[derive(Serialize)]
struct DialPressEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: DialPressPayload,
}

pub async fn dial_rotate(device: &str, index: u8, ticks: i16) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;
	let instance = match get_instance(device, "Encoder", index, 0, &mut locks).await? {
		Some(instance) => instance,
		None => return Ok(()),
	};

	send_to_plugin(
		&instance.action.plugin,
		&DialRotateEvent {
			event: "dialRotate",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: DialRotatePayload {
				settings: instance.settings.clone(),
				coordinates: Coordinates {
					row: instance.context.position / 3,
					column: instance.context.position % 3,
				},
				ticks,
				pressed: false,
			},
		},
	)
	.await
}

pub async fn dial_press(device: &str, event: &'static str, index: u8) -> Result<(), anyhow::Error> {
	let mut locks = lock_mutexes().await;
	let instance = match get_instance(device, "Encoder", index, 0, &mut locks).await? {
		Some(instance) => instance,
		None => return Ok(()),
	};

	send_to_plugin(
		&instance.action.plugin,
		&DialPressEvent {
			event,
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: DialPressPayload {
				controller: "Encoder",
				settings: instance.settings.clone(),
				coordinates: Coordinates {
					row: instance.context.position / 3,
					column: instance.context.position % 3,
				},
			},
		},
	)
	.await
}
