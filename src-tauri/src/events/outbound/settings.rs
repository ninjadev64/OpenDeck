use super::{Coordinates, send_to_plugin};

use crate::shared::ActionContext;

use serde::Serialize;

#[derive(Serialize)]
pub struct DidReceiveSettingsPayload {
	settings: serde_json::Value,
	coordinates: Coordinates
}

#[derive(Serialize)]
pub struct DidReceiveSettings {
	event: String,
	action: String,
	context: ActionContext,
	device: String,
	payload: DidReceiveSettingsPayload
}

pub async fn did_receive_settings(context: crate::shared::ActionContext, instance: &crate::shared::ActionInstance) -> Result<(), anyhow::Error> {
	send_to_plugin(&instance.action.plugin, DidReceiveSettings {
		event: "didReceiveSettings".to_owned(),
		action: instance.action.uuid.clone(),
		context: context.clone(),
		device: context.device.clone(),
		payload: DidReceiveSettingsPayload {
			settings: instance.settings.clone(),
			coordinates: Coordinates {
				row: instance.context.position / 3,
				column: instance.context.position % 3
			}
		}
	}).await
}
