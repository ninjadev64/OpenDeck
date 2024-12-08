use crate::events::outbound::{encoder, keypad};

use std::collections::HashMap;

use base64::Engine as _;
use elgato_streamdeck::{info, AsyncStreamDeck, DeviceStateUpdate};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

static ELGATO_DEVICES: Lazy<RwLock<HashMap<String, AsyncStreamDeck>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn update_image(context: &crate::shared::Context, image: Option<&str>) -> Result<(), anyhow::Error> {
	if let Some(device) = ELGATO_DEVICES.read().await.get(&context.device) {
		if let Some(image) = image {
			let data = image.split_once(',').unwrap().1;
			let bytes = base64::engine::general_purpose::STANDARD.decode(data)?;
			device.set_button_image(context.position, image::load_from_memory(&bytes)?).await?;
		} else {
			device.clear_button_image(context.position).await?;
		}
		device.flush().await?;
	}
	Ok(())
}

pub async fn clear_screen(id: &str) -> Result<(), anyhow::Error> {
	if let Some(device) = ELGATO_DEVICES.read().await.get(id) {
		device.clear_all_button_images().await?;
		device.flush().await?;
	}
	Ok(())
}

pub async fn set_brightness(brightness: u8) {
	for (_id, device) in ELGATO_DEVICES.read().await.iter() {
		let _ = device.set_brightness(brightness.clamp(0, 100)).await;
		let _ = device.flush().await;
	}
}

pub(super) async fn init(device: AsyncStreamDeck) {
	let kind = device.kind();
	let device_type = match kind.product_id() {
		info::PID_STREAMDECK_ORIGINAL | info::PID_STREAMDECK_ORIGINAL_V2 | info::PID_STREAMDECK_MK2 => 0,
		info::PID_STREAMDECK_MINI | info::PID_STREAMDECK_MINI_MK2 => 1,
		info::PID_STREAMDECK_XL | info::PID_STREAMDECK_XL_V2 => 2,
		info::PID_STREAMDECK_PEDAL => 5,
		info::PID_STREAMDECK_PLUS => 7,
		_ => 7,
	};
	let _ = device.clear_all_button_images().await;
	let device_id = format!("sd-{}", device.serial_number().await.unwrap().chars().filter(|c| c.is_alphanumeric()).collect::<String>());
	crate::events::inbound::devices::register_device(
		"",
		crate::events::inbound::PayloadEvent {
			payload: crate::shared::DeviceInfo {
				id: device_id.clone(),
				plugin: String::new(),
				name: device.product().await.unwrap(),
				rows: kind.row_count(),
				columns: kind.column_count(),
				encoders: kind.encoder_count(),
				r#type: device_type,
			},
		},
	)
	.await
	.unwrap();

	let reader = device.get_reader();
	ELGATO_DEVICES.write().await.insert(device_id.clone(), device);
	loop {
		let updates = match reader.read(100.0).await {
			Ok(updates) => updates,
			Err(_) => break,
		};
		for update in updates {
			match match update {
				DeviceStateUpdate::ButtonDown(key) => keypad::key_down(&device_id, key).await,
				DeviceStateUpdate::ButtonUp(key) => keypad::key_up(&device_id, key).await,
				DeviceStateUpdate::EncoderTwist(dial, ticks) => encoder::dial_rotate(&device_id, dial, ticks.into()).await,
				DeviceStateUpdate::EncoderDown(dial) => encoder::dial_press(&device_id, "dialDown", dial).await,
				DeviceStateUpdate::EncoderUp(dial) => encoder::dial_press(&device_id, "dialUp", dial).await,
				_ => Ok(()),
			} {
				Ok(_) => (),
				Err(error) => log::warn!("Failed to process device event {:?}: {}", update, error),
			}
		}
	}

	crate::events::inbound::devices::deregister_device("", crate::events::inbound::PayloadEvent { payload: device_id })
		.await
		.unwrap();
}

/// Attempt to initialise all connected devices.
pub fn initialise_devices() {
	// Iterate through detected Elgato devices and attempt to register them.
	match elgato_streamdeck::new_hidapi() {
		Ok(hid) => {
			for (kind, serial) in elgato_streamdeck::asynchronous::list_devices_async(&hid) {
				match elgato_streamdeck::AsyncStreamDeck::connect(&hid, kind, &serial) {
					Ok(device) => {
						tokio::spawn(init(device));
					}
					Err(error) => log::warn!("Failed to connect to Elgato device: {}", error),
				}
			}
		}
		Err(error) => log::warn!("Failed to initialise hidapi: {}", error),
	}
}
