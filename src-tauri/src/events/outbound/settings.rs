use super::{send_to_plugin, send_to_property_inspector, Coordinates};

use crate::shared::ActionContext;

use serde::Serialize;

#[derive(Serialize)]
struct DidReceiveSettingsPayload {
	settings: serde_json::Value,
	coordinates: Coordinates,
}

#[derive(Serialize)]
struct DidReceiveSettingsEvent {
	event: &'static str,
	action: String,
	context: ActionContext,
	device: String,
	payload: DidReceiveSettingsPayload,
}

#[derive(Serialize)]
struct DidReceiveGlobalSettingsPayload {
	settings: serde_json::Value,
}

#[derive(Serialize)]
struct DidReceiveGlobalSettingsEvent {
	event: &'static str,
	payload: DidReceiveGlobalSettingsPayload,
}

pub async fn did_receive_settings(instance: &crate::shared::ActionInstance, to_property_inspector: bool) -> Result<(), anyhow::Error> {
	let data = DidReceiveSettingsEvent {
		event: "didReceiveSettings",
		action: instance.action.uuid.clone(),
		context: instance.context.clone(),
		device: instance.context.device.clone(),
		payload: DidReceiveSettingsPayload {
			settings: instance.settings.clone(),
			coordinates: Coordinates {
				row: instance.context.position / 3,
				column: instance.context.position % 3,
			},
		},
	};
	if to_property_inspector {
		send_to_property_inspector(&instance.context, &data).await
	} else {
		send_to_plugin(&instance.action.plugin, &data).await
	}
}

pub async fn did_receive_global_settings(context: &str, to_property_inspector: bool) -> Result<(), anyhow::Error> {
	let app = crate::APP_HANDLE.get().unwrap();

	let settings_dir = app.path_resolver().app_config_dir().unwrap().join("settings/");
	let path = settings_dir.join(format!("{}.json", context));
	let settings: serde_json::Value = match std::fs::read(path) {
		Ok(contents) => serde_json::from_slice(&contents)?,
		Err(_) => serde_json::Value::Object(serde_json::Map::new()),
	};

	let data = DidReceiveGlobalSettingsEvent {
		event: "didReceiveGlobalSettings",
		payload: DidReceiveGlobalSettingsPayload { settings },
	};

	if to_property_inspector {
		let profile_stores = crate::store::profiles::PROFILE_STORES.lock().await;
		for context in profile_stores.all_from_plugin(context) {
			send_to_property_inspector(&context, &data).await?;
		}
	} else {
		send_to_plugin(context, &data).await?;
	}

	Ok(())
}
