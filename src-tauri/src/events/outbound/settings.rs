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

#[derive(Serialize)]
pub struct DidReceiveGlobalSettingsPayload {
	settings: serde_json::Value
}

#[derive(Serialize)]
pub struct DidReceiveGlobalSettings {
	event: String,
	payload: DidReceiveGlobalSettingsPayload
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

pub async fn did_receive_global_settings(context: &str) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.lock().await;
	let app = app.as_ref().unwrap();

	let settings_dir = app.path_resolver().app_config_dir().unwrap().join("settings/");
	let path = settings_dir.join(format!("{}.json", context));
	let settings: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;

	send_to_plugin(context, DidReceiveGlobalSettings {
		event: "didReceiveGlobalSettings".to_owned(),
		payload: DidReceiveGlobalSettingsPayload {
			settings
		}
	}).await
}
