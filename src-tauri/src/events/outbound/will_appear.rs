use super::{send_to_plugin, GenericInstancePayload};

use crate::shared::{ActionContext, ActionInstance};

#[derive(serde::Serialize)]
struct AppearEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: GenericInstancePayload,
}

pub async fn will_appear(instance: &ActionInstance, multi_action: bool) -> Result<(), anyhow::Error> {
	send_to_plugin(
		&instance.action.plugin,
		&AppearEvent {
			event: "willAppear",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(instance, multi_action),
		},
	)
	.await?;

	Ok(())
}

pub async fn will_disappear(instance: &ActionInstance, multi_action: bool) -> Result<(), anyhow::Error> {
	send_to_plugin(
		&instance.action.plugin,
		&AppearEvent {
			event: "willDisappear",
			action: instance.action.uuid.clone(),
			context: instance.context.clone(),
			device: instance.context.device.clone(),
			payload: GenericInstancePayload::new(instance, multi_action),
		},
	)
	.await?;

	if instance.context.device.starts_with("sd-") {
		if let Err(error) = crate::devices::elgato::clear_image(&(&instance.context).into()).await {
			log::warn!("Failed to clear device image at context {}: {}", instance.context, error);
		}
	}

	Ok(())
}
