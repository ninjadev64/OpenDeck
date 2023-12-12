pub mod keypad;
pub mod encoder;

use super::SOCKETS;

use crate::APP_HANDLE;
use crate::devices::DEVICES;
use crate::shared::ActionInstance;
use crate::store::profiles::{DEVICE_STORES, PROFILE_STORES};

use serde::Serialize;
use futures_util::SinkExt;

#[derive(Serialize)]
struct Coordinates {
	row: u8,
	column: u8
}

async fn send_to_plugin(plugin: &str, data: impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&data).unwrap());
	SOCKETS.lock().await.get_mut(plugin).unwrap().send(message).await?;
	Ok(())
}

async fn get_instance(device: &str, index: u8, controller: &str) -> Result<Option<ActionInstance>, anyhow::Error> {
	let app = APP_HANDLE.lock().await;
	let app = app.as_ref().unwrap();

	let mut device_stores = DEVICE_STORES.lock().await;
	let selected_profile = &device_stores.get_device_store(device, app)?.value.selected_profile;

	let devices = DEVICES.lock().await;
	let device = devices.get(device).unwrap();

	let mut profile_stores = PROFILE_STORES.lock().await;
	let profile = &profile_stores.get_profile_store(device, selected_profile, app)?.value;

	let configured = match controller {
		"Encoder" => profile.sliders[index as usize].as_ref(),
		_ => profile.keys[index as usize].as_ref()
	};
	match configured {
		Some(configured) => Ok(Some(configured.clone())),
		None => Ok(None)
	}
}
