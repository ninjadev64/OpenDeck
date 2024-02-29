mod prontokey;
pub mod elgato;

use std::collections::HashMap;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use serde::Serialize;
use log::warn;

/// Metadata of a device.
#[derive(Clone, Serialize)]
pub struct DeviceInfo {
	pub id: String,
	pub name: String,
	pub rows: u8,
	pub columns: u8,
	pub sliders: u8,
	pub r#type: u8
}

lazy_static! {
	pub static ref DEVICES: Mutex<HashMap<String, DeviceInfo>> = Mutex::new(HashMap::new());
}

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
					Ok(device) => { tokio::spawn(elgato::init(device)); },
					Err(error) => warn!("Failed to connect to Elgato device: {}", error)
				}
			}
		},
		Err(error) => warn!("Failed to initialise hidapi: {}", error)
	}


	// Create a virtual device for testing without a physical device.
	tokio::spawn(async move {
		let mut devices = DEVICES.lock().await;
		let device = DeviceInfo {
			id: "virtual".to_owned(),
			name: "Virtual device".to_owned(),
			rows: 3,
			columns: 3,
			sliders: 2,
			r#type: 7
		};
		devices.insert("virtual".to_owned(), device);
	});
}

async fn register_device(id: String, device: DeviceInfo) {
	crate::events::outbound::devices::device_did_connect(&id, (&device).into()).await.ok();
	DEVICES.lock().await.insert(id, device);
}

async fn unregister_device(id: String) {
	crate::events::outbound::devices::device_did_disconnect(&id).await.ok();
	DEVICES.lock().await.remove(&id);
}
