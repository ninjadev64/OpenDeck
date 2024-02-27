use super::DeviceInfo;

use crate::events::outbound;

use std::time::Duration;

use log::{warn, error};

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ProntoKeyMessage {
	Registration { a: String },
	Update { k: [u8; 9], s: [u16; 2] }
}

/// Attempt to open a serial connection with the device and handle incoming data.
pub async fn init(port: String) {
	let mut initialised = false;
	let mut device_id = "".to_owned();

	let mut last_keys: [u8; 9] = [0; 9];
	let mut last_sliders: [u16; 2] = [0; 2];

	let mut port = match
		serialport::new(port, 115200)
		.timeout(Duration::from_millis(10))
		.open()
	{
		Ok(p) => p,
		Err(error) => {
			error!("Failed to open serial port: {}", error);
			return;
		}
	};
	let _ = port.write("#".as_bytes());

	let serial_buf: &mut [u8] = &mut [0; 64];
	let mut holding_string = String::from("");

	loop {
		match port.read(serial_buf) {
			Ok(t) => {
				match std::str::from_utf8(&serial_buf[..t]) {
					Ok(data) => holding_string += data,
					Err(_) => break
				}

				// Iterate through JSON objects received from device that are being held in the buffer.
				while holding_string.contains('}') {
					let index = holding_string.find('}').unwrap_or_default();
					let chunk = holding_string[..=index].trim();
					let data: ProntoKeyMessage = match serde_json::from_str(chunk) {
						Ok(data) => data,
						Err(_) => continue
					};
					holding_string = holding_string[(index + 1)..].to_owned();

					if let ProntoKeyMessage::Registration { a: address } = data {
						// If the device is uninitialised, attempt to read its MAC address and initialise.
						if initialised { continue }
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
					} else if let ProntoKeyMessage::Update { k: keys, s: sliders } = data {
						if !initialised { continue }

						// Handle key presses and releases.
						for (index, value) in keys.iter().enumerate() {
							if *value != last_keys[index] {
								if *value == 0 {
									let _ = outbound::keypad::key_up(&device_id, index as u8).await;
								} else {
									let _ = outbound::keypad::key_down(&device_id, index as u8).await;
								}
								last_keys[index] = *value;
							}
						}

						// Handle slider value changes.
						for (index, value) in sliders.iter().enumerate() {
							if *value != last_sliders[index] {
								let change = ((*value as i32 - last_sliders[index] as i32) / (4095 / 192)) as i16;
								last_sliders[index] = *value;
								let _ = outbound::encoder::dial_rotate(&device_id, index as u8, change).await;
							}
						}
					}
				}
			},
			Err(ref error) if error.kind() == std::io::ErrorKind::TimedOut => (),
			Err(error) => warn!("Failed to read serial message from ProntoKey device: {}", error)
		}
		tokio::time::sleep(Duration::from_millis(10)).await;
	}
}
