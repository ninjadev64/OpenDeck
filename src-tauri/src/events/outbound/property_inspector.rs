use crate::shared::ActionContext;

use serde::Serialize;

#[derive(Serialize)]
pub struct SendTo {
	event: String,
	action: String,
	context: ActionContext,
	payload: serde_json::Value
}

pub async fn send_to_property_inspector(context: ActionContext, message: serde_json::Value) -> Result<(), anyhow::Error> {
	if let Some(instance) = crate::store::profiles::get_instance(
		&context.device,
		context.position,
		&context.controller
	).await? {
		super::send_to_property_inspector(&context, &SendTo {
			event: "sendToPropertyInspector".to_owned(),
			action: instance.action.uuid.clone(),
			context: context.clone(),
			payload: message
		}).await?;
	}

	Ok(())
}

pub async fn send_to_plugin(context: ActionContext, message: serde_json::Value) -> Result<(), anyhow::Error> {
	if let Some(instance) = crate::store::profiles::get_instance(
		&context.device,
		context.position,
		&context.controller
	).await? {
		super::send_to_plugin(&instance.action.plugin, &SendTo {
			event: "sendToPlugin".to_owned(),
			action: instance.action.uuid.clone(),
			context,
			payload: message
		}).await?;
	}

	Ok(())
}
