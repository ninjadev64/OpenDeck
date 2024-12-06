pub mod elgato;

use crate::store::profiles::get_device_profiles;

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_inline_default::serde_inline_default;
use tokio::sync::RwLock;

use log::{error, warn};
use serde::{Deserialize, Serialize};

/// Metadata of a device.
#[serde_inline_default]
#[derive(Clone, Deserialize, Serialize)]
pub struct DeviceInfo {
	pub id: String,
	#[serde_inline_default(String::new())]
	pub plugin: String,
	pub name: String,
	pub rows: u8,
	pub columns: u8,
	pub encoders: u8,
	pub r#type: u8,
}

pub static DEVICES: Lazy<RwLock<HashMap<String, DeviceInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));
pub static DEVICE_NAMESPACES: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Attempt to initialise all connected devices.
pub fn initialise_devices() {
	// Iterate through detected Elgato devices and attempt to register them.
	match elgato_streamdeck::new_hidapi() {
		Ok(hid) => {
			for (kind, serial) in elgato_streamdeck::asynchronous::list_devices_async(&hid) {
				match elgato_streamdeck::AsyncStreamDeck::connect(&hid, kind, &serial) {
					Ok(device) => {
						tokio::spawn(elgato::init(device));
					}
					Err(error) => warn!("Failed to connect to Elgato device: {}", error),
				}
			}
		}
		Err(error) => warn!("Failed to initialise hidapi: {}", error),
	}

	// Create a virtual device for testing without a physical device.
	#[cfg(debug_assertions)]
	tokio::spawn(async move {
		let device = DeviceInfo {
			id: "virtual".to_owned(),
			plugin: String::new(),
			name: "Virtual device".to_owned(),
			rows: 3,
			columns: 5,
			encoders: 0,
			r#type: 0,
		};
		register_device(device).await;
	});
}

pub async fn register_device(device: DeviceInfo) {
	if let Ok(profiles) = get_device_profiles(&device.id) {
		let mut profile_stores = crate::store::profiles::PROFILE_STORES.write().await;
		for profile in profiles {
			// This is called to initialise the store for each profile when the device is registered.
			if let Err(e) = profile_stores.get_profile_store_mut(&device, &profile).await {
				error!("{}", e);
			}
		}
	}
	crate::events::outbound::devices::device_did_connect(&device.id, (&device).into()).await.ok();
	DEVICES.write().await.insert(device.id.clone(), device);
	crate::events::frontend::update_devices().await;
}

pub async fn unregister_device(id: String) {
	crate::events::outbound::devices::device_did_disconnect(&id).await.ok();
	DEVICES.write().await.remove(&id);
	crate::events::frontend::update_devices().await;
}
