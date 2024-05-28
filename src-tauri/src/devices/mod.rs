pub mod elgato;
mod prontokey;

use crate::store::profiles::get_device_profiles;

use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use log::warn;
use serde::Serialize;

/// Metadata of a device.
#[derive(Clone, Serialize)]
pub struct DeviceInfo {
	pub id: String,
	pub name: String,
	pub rows: u8,
	pub columns: u8,
	pub sliders: u8,
	pub r#type: u8,
}

pub static DEVICES: Lazy<RwLock<HashMap<String, DeviceInfo>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Attempt to initialise all connected devices.
pub fn initialise_devices() {
	// Iterate through available serial ports and attempt to register them as ProntoKey devices.
	for port in serialport::available_ports().unwrap_or_default() {
		if let serialport::SerialPortType::UsbPort(info) = port.port_type {
			if info.vid == 0x10c4 && info.pid == 0xea60 {
				tokio::spawn(prontokey::init(port.port_name));
			}
		}
	}

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
			name: "Virtual device".to_owned(),
			rows: 3,
			columns: 3,
			sliders: 2,
			r#type: 7,
		};
		register_device("virtual".to_owned(), device).await;
	});
}

async fn register_device(id: String, device: DeviceInfo) {
	let app = crate::APP_HANDLE.get().unwrap();
	if let Ok(profiles) = get_device_profiles(&device.id, app) {
		let mut profile_stores = crate::store::profiles::PROFILE_STORES.write().await;
		for profile in profiles {
			// This is called to initialise the store for each profile when the device is registered.
			let _ = profile_stores.get_profile_store_mut(&device, &profile, app);
		}
	}
	crate::events::outbound::devices::device_did_connect(&id, (&device).into()).await.ok();
	DEVICES.write().await.insert(id, device);
	crate::events::frontend::update_devices().await;
}

async fn unregister_device(id: String) {
	crate::events::outbound::devices::device_did_disconnect(&id).await.ok();
	DEVICES.write().await.remove(&id);
	crate::events::frontend::update_devices().await;
}
