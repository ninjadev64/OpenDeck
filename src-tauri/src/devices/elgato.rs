use elgato_streamdeck::{StreamDeck, info};

pub async fn init(device: StreamDeck) {
	let kind = device.kind();
	let device_type = match kind.product_id() {
		info::PID_STREAMDECK_ORIGINAL | info::PID_STREAMDECK_ORIGINAL_V2 | info::PID_STREAMDECK_MK2 => 0,
		info::PID_STREAMDECK_MINI | info::PID_STREAMDECK_MINI_MK2 => 1,
		info::PID_STREAMDECK_XL | info::PID_STREAMDECK_XL_V2 => 2,
		info::PID_STREAMDECK_PEDAL => 5,
		info::PID_STREAMDECK_PLUS => 7,
		_ => 7
	};
	let device_id = format!("sd-{}", device.serial_number().unwrap());
	super::DEVICES.lock().await.insert(device_id.clone(), super::DeviceInfo {
		id: device_id.clone(),
		name: device.product().unwrap(),
		rows: kind.row_count(),
		columns: kind.column_count(),
		sliders: kind.encoder_count(),
		r#type: device_type
	});
}
