use super::BaseDevice;

use std::thread;
use std::time::Duration;

use serde_json::Value;

/// A representation of a ProntoKey device.
pub struct ProntoKeyDevice {
	address: String
}

impl BaseDevice for ProntoKeyDevice {
	fn num_dials(&self) -> u8 { 2 }
	fn num_rows(&self) -> u8 { 3 }
	fn num_columns(&self) -> u8 { 3 }

	fn id(&self) -> String {
		format!("pk-{}", self.address)
	}
	fn name(&self) -> String {
		String::from("ProntoKey")
	}
	fn r#type(&self) -> u8 { 7 }

	fn key_down(&self, key: u8) {
		println!("{} down {}", &self.address, key);
	}
	fn key_up(&self, key: u8) {
		println!("{} up {}", &self.address, key);
	}
	fn dial_rotate(&self, dial: u8, ticks: i16) {
		println!("{} rotate {} by {}", &self.address, dial, ticks);
	}
}

impl ProntoKeyDevice {
	/// Attempt to open a serial connection with the device and handle incoming data.
	pub async fn init(port: String) {
		let mut device: Option<ProntoKeyDevice> = Option::None;
		let mut last_key: u8 = 0;
		let mut last_sliders: Vec<i16> = vec![0; 2];

		let mut port = match
			serialport::new(port, 57600)
			.timeout(Duration::from_millis(10))
			.open()
		{
			Ok(p) => p,
			Err(error) => panic!("Failed to open serial port: {}", error)
		};
		let _ = port.write("register".as_bytes()); 

		let mut serial_buf: Vec<u8> = vec![0; 1024];
		let mut holding_string = String::from("");

		loop {
			match port.read(serial_buf.as_mut_slice()) {
				Ok(t) => {
					match std::str::from_utf8(&serial_buf[..t]) {
						Ok(data) => holding_string += data,
						Err(_) => break
					}
					// Iterate through JSON objects received from device that are being held in the buffer.
					while holding_string.contains('}') {
						let index = holding_string.find('}').unwrap_or_default();
						let chunk = holding_string[..=index].trim();
						let j: Value = match serde_json::from_str(chunk) {
							Ok(data) => data,
							Err(_) => continue
						};
						holding_string = holding_string[(index + 1)..].to_owned();
						
						// If the device is uninitialised, attempt to read its MAC address and initialise.
						if device.is_none() {
							if let Value::String(address) = &j["address"] {
								device = Some(ProntoKeyDevice { address: address.clone() });
								if let Some(device) = &device {
									super::DEVICES.lock().unwrap().push(super::DeviceInfo::new(device));
								}
							}
							continue;
						}

						let device = device.as_ref().unwrap();

						// Handle key presses and releases.
						if let Value::Number(num) = &j["key"] {
							match num.as_u64().unwrap_or_default() as u8 {
								0 => device.key_up(last_key),
								val => {
									device.key_down(val);
									last_key = val;
								}
							}
						}

						// Handle slider value changes.
						if let Value::Number(val) = &j["slider0"] {
							let val: i16 = match val.as_i64() {
								Some(v) => v as i16,
								_ => last_sliders[0]
							};
							device.dial_rotate(0, val - last_sliders[0]);
							last_sliders[0] = val;
						}
						if let Value::Number(val) = &j["slider1"] {
							let val: i16 = match val.as_i64() {
								Some(v) => v as i16,
								_ => last_sliders[1]
							};
							device.dial_rotate(1, val - last_sliders[1]);
							last_sliders[1] = val;
						}
					}
				},
				Err(ref error) if error.kind() == std::io::ErrorKind::TimedOut => (),
				Err(error) => eprintln!("{:?}", error)
			}
			thread::sleep(Duration::from_millis(10));
		}
	}
}
