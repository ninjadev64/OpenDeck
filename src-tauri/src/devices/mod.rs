mod prontokey;

use std::collections::HashMap;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use serde::Serialize;

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
				tokio::spawn(prontokey::ProntoKeyDevice::init(port.port_name));
			}
		}
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
