use super::{Coordinates, send_to_plugin};

use crate::shared::ActionContext;
use crate::store::profiles::get_instance;

use serde::Serialize;

#[derive(Serialize)]
struct DialRotatePayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
	ticks: i16,
	pressed: bool
}

#[derive(Serialize)]
struct DialRotateEvent {
	action: String,
	event: String,
	context: ActionContext,
	device: String,
	payload: DialRotatePayload
}

pub async fn dial_rotate(device: String, index: u8, ticks: i16) -> Result<(), anyhow::Error> {
	let instance = match get_instance(&device, index, "Encoder").await? {
		Some(instance) => instance,
		None => return Ok(())
	};

	send_to_plugin(&instance.action.plugin, &DialRotateEvent {
		action: instance.action.uuid.clone(),
		event: "dialRotate".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: DialRotatePayload {
			settings: instance.settings.clone(),
			coordinates: Coordinates {
				row: instance.context.position / 3,
				column: instance.context.position % 3
			},
			ticks,
			pressed: false
		}
	}).await
}
