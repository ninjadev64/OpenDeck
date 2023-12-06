mod prontokey;

use std::collections::HashMap;

use lazy_static::lazy_static;
use std::sync::Mutex;

use serde::Serialize;

/// A trait implemented by all supported devices.
pub trait BaseDevice {
	fn num_sliders(&self) -> u8;
	fn num_rows(&self) -> u8;
	fn num_columns(&self) -> u8;
	fn num_keys(&self) -> u8 {
		self.num_rows() * self.num_columns()
	}
	
	fn id(&self) -> String;
	fn name(&self) -> String;
	fn r#type(&self) -> u8;

	fn key_down(&self, key: u8);
	fn key_up(&self, key: u8);
	fn dial_rotate(&self, dial: u8, ticks: i16);
}

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

impl DeviceInfo {
	fn new(device: &impl BaseDevice) -> DeviceInfo {
		DeviceInfo {
			id: device.id(),
			name: device.name(),
			rows: device.num_rows(),
			columns: device.num_columns(),
			sliders: device.num_sliders(),
			r#type: device.r#type()
		}
	}
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
}
