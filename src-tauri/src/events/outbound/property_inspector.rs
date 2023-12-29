use crate::shared::ActionContext;

use futures_util::SinkExt as _;
use serde::Serialize;

#[derive(Serialize)]
pub struct SendTo {
	event: String,
	action: String,
	context: ActionContext,
	payload: serde_json::Value
}

pub async fn send_to_property_inspector(context: ActionContext, message: serde_json::Value) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match context.controller.as_str() {
		"Encoder" => profile.sliders[context.position as usize].as_mut(),
		_ => profile.keys[context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&SendTo {
			event: "sendToPropertyInspector".to_owned(),
			action: instance.action.uuid.clone(),
			context: context.clone(),
			payload: message
		}).unwrap());
		crate::events::PROPERTY_INSPECTOR_SOCKETS.lock().await.get_mut(&context.to_string()).unwrap().send(message).await?;
	}

	Ok(())
}

pub async fn send_to_plugin(context: ActionContext, message: serde_json::Value) -> Result<(), anyhow::Error> {
	let (
		app,
		mut device_stores,
		devices,
		mut profile_stores
	) = crate::store::profiles::lock_mutexes().await;

	let selected_profile = &device_stores.get_device_store(&context.device, app.as_ref().unwrap())?.value.selected_profile;
	let device = devices.get(&context.device).unwrap();
	let store = profile_stores.get_profile_store(device, selected_profile, app.as_ref().unwrap())?;
	let profile = &mut store.value;

	let instance = match context.controller.as_str() {
		"Encoder" => profile.sliders[context.position as usize].as_mut(),
		_ => profile.keys[context.position as usize].as_mut()
	};

	if let Some(instance) = instance {
		super::send_to_plugin(&instance.action.plugin, SendTo {
			event: "sendToPlugin".to_owned(),
			action: instance.action.uuid.clone(),
			context,
			payload: message
		}).await?;
	}

	Ok(())
}
