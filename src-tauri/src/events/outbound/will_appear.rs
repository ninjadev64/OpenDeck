use super::{GenericInstancePayload, send_to_plugin};

use crate::shared::{ActionContext, ActionInstance};

#[derive(serde::Serialize)]
struct AppearEvent {
	action: String,
	event: String,
	context: ActionContext,
	device: String,
	payload: GenericInstancePayload
}

pub async fn will_appear(instance: &ActionInstance) -> Result<(), anyhow::Error> {
	send_to_plugin(&instance.action.plugin, &AppearEvent {
		action: instance.action.uuid.clone(),
		event: "willAppear".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: GenericInstancePayload::new(&instance)
	}).await?;

	Ok(())
}

pub async fn will_disappear(instance: &ActionInstance) -> Result<(), anyhow::Error> {
	send_to_plugin(&instance.action.plugin, &AppearEvent {
		action: instance.action.uuid.clone(),
		event: "willDisappear".to_owned(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: GenericInstancePayload::new(&instance)
	}).await?;

	Ok(())
}
