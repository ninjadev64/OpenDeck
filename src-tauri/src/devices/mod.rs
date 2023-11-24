mod prontokey;

use lazy_static::lazy_static;
use std::sync::Mutex;

use serde::Serialize;

/// A trait implemented by all supported devices.
pub trait BaseDevice {
	fn num_dials(&self) -> u8;
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


/// The dimensions of a device.
#[derive(Clone, Serialize)]
pub struct DeviceSize {
	pub rows: u8,
	pub columns: u8
}

/// Metadata of a device.
#[derive(Clone, Serialize)]
pub struct DeviceInfo {
	pub id: String,
	pub name: String,
	pub size: DeviceSize,
	pub r#type: u8
}

impl DeviceInfo {
	fn new(device: &impl BaseDevice) -> DeviceInfo {
		DeviceInfo {
			id: device.id(),
			name: device.name(),
			size: DeviceSize {
				rows: device.num_rows(),
				columns: device.num_columns()
			},
			r#type: device.r#type()
		}
	}
}

lazy_static! {
	pub static ref DEVICES: Mutex<Vec<DeviceInfo>> = Mutex::new(vec![]);
}

/// Attempt to initialise all connected devices.
pub fn initialise_devices() {
	// Iterate through available serial ports and attempt to register them as ProntoKey devices.
	for port in serialport::available_ports().unwrap_or_default() {
		match port.port_type {
			serialport::SerialPortType::UsbPort(info) => {
				if info.vid == 0x10c4 && info.pid == 0xea60 {
					tokio::spawn(prontokey::ProntoKeyDevice::init(port.port_name));
				}
			}
			_ => {}
		}
	}
}
