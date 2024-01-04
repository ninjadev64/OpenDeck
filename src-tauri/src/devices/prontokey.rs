use super::DeviceInfo;

use crate::events::outbound;

use std::thread;
use std::time::Duration;

use serde_json::Value;
use log::{warn, error};

pub struct ProntoKeyDevice {}

impl ProntoKeyDevice {
	/// Attempt to open a serial connection with the device and handle incoming data.
	pub async fn init(port: String) {
		let mut initialised = false;
		let mut device_id = "".to_owned();
		let mut last_key: u8 = 0;
		let mut last_sliders: Vec<i16> = vec![0; 2];

		let mut port = match
			serialport::new(port, 57600)
			.timeout(Duration::from_millis(10))
			.open()
		{
			Ok(p) => p,
			Err(error) => {
				error!("Failed to open serial port: {}", error);
				panic!()
			}
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
						if !initialised {
							if let Value::String(address) = &j["address"] {
								initialised = true;
								device_id = format!("pk-{}", address);
								super::DEVICES.lock().await.insert(device_id.clone(), DeviceInfo {
									id: device_id.clone(),
									name: "ProntoKey".to_owned(),
									rows: 3,
									columns: 3,
									sliders: 2,
									r#type: 7
								});
							}
							continue;
						}

						// Handle key presses and releases.
						if let Value::Number(num) = &j["key"] {
							match num.as_u64().unwrap_or_default() as u8 {
								0 => {
									let _ = outbound::keypad::key_up(&device_id, last_key - 1).await;
								},
								val => {
									let _ = outbound::keypad::key_down(&device_id, val - 1).await;
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
							let _ = outbound::encoder::dial_rotate(&device_id, 0, val - last_sliders[0]).await;
							last_sliders[0] = val;
						}
						if let Value::Number(val) = &j["slider1"] {
							let val: i16 = match val.as_i64() {
								Some(v) => v as i16,
								_ => last_sliders[1]
							};
							let _ = outbound::encoder::dial_rotate(&device_id, 1, val - last_sliders[1]).await;
							last_sliders[1] = val;
						}
					}
				},
				Err(ref error) if error.kind() == std::io::ErrorKind::TimedOut => (),
				Err(error) => warn!("Failed to decode serial message from ProntoKey device: {}", error)
			}
			thread::sleep(Duration::from_millis(10));
		}
	}
}
