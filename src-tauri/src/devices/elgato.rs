use crate::events::outbound::{keypad, encoder};

use std::collections::HashMap;

use elgato_streamdeck::{AsyncStreamDeck, DeviceStateUpdate, info};

use lazy_static::lazy_static;
use tokio::sync::Mutex;
use base64::Engine as _;

lazy_static! {
	static ref ELGATO_DEVICES: Mutex<HashMap<String, AsyncStreamDeck>> = Mutex::new(HashMap::new());
}

pub async fn update_image(context: &crate::shared::ActionContext, url: &str) -> Result<(), anyhow::Error> {
	if let Some(device) = ELGATO_DEVICES.lock().await.get(&context.device) {
		let data = url.split_once(',').unwrap().1;
		let bytes = base64::engine::general_purpose::STANDARD.decode(data)?;
		device.set_button_image(context.position, image::load_from_memory(&bytes)?).await?;
	}
	Ok(())
}

pub async fn clear_image(context: &crate::shared::ActionContext) -> Result<(), anyhow::Error> {
	if let Some(device) = ELGATO_DEVICES.lock().await.get(&context.device) {
		device.clear_button_image(context.position).await?;
	}
	Ok(())
}

pub(super) async fn init(device: AsyncStreamDeck) {
	let kind = device.kind();
	let device_type = match kind.product_id() {
		info::PID_STREAMDECK_ORIGINAL | info::PID_STREAMDECK_ORIGINAL_V2 | info::PID_STREAMDECK_MK2 => 0,
		info::PID_STREAMDECK_MINI | info::PID_STREAMDECK_MINI_MK2 => 1,
		info::PID_STREAMDECK_XL | info::PID_STREAMDECK_XL_V2 => 2,
		info::PID_STREAMDECK_PEDAL => 5,
		info::PID_STREAMDECK_PLUS => 7,
		_ => 7
	};
	let device_id = format!("sd-{}", device.serial_number().await.unwrap());
	super::DEVICES.lock().await.insert(device_id.clone(), super::DeviceInfo {
		id: device_id.clone(),
		name: device.product().await.unwrap(),
		rows: kind.row_count(),
		columns: kind.column_count(),
		sliders: kind.encoder_count(),
		r#type: device_type
	});

	let reader = device.get_reader();
	ELGATO_DEVICES.lock().await.insert(device_id.clone(), device);
	loop {
		let updates = match reader.read(100.0).await {
			Ok(updates) => updates,
			Err(_) => continue
		};
		for update in updates {
			match match update {
				DeviceStateUpdate::ButtonDown(key) => keypad::key_down(&device_id, key).await,
				DeviceStateUpdate::ButtonUp(key) => keypad::key_up(&device_id, key).await,
				DeviceStateUpdate::EncoderTwist(dial, ticks) => encoder::dial_rotate(&device_id, dial, ticks.into()).await,
				DeviceStateUpdate::EncoderDown(dial) => encoder::dial_press(&device_id, "dialDown", dial).await,
				DeviceStateUpdate::EncoderUp(dial) => encoder::dial_press(&device_id, "dialUp", dial).await,
				_ => Ok(())
			} {
				Ok(_) => (),
				Err(error) => log::warn!("Failed to process device event {:?}: {}", update, error)
			}
		}
	}
}
