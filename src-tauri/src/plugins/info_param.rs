use serde::Serialize;

// Structs that make up the Info parameter passed to plugins during the registration procedure.

#[allow(non_snake_case)]
#[derive(Serialize)]
pub(super) struct ApplicationInfo {
	pub font: String,
	pub language: String,
	pub platform: String,
	pub platformVersion: String,
	pub version: String
}

#[derive(Serialize)]
pub(super) struct PluginInfo {
	pub uuid: String,
	pub version: String
}

#[allow(non_snake_case)]
#[derive(Serialize)]
pub(super) struct ColoursInfo {
	pub buttonPressedBackgroundColor: String,
	pub buttonPressedBorderColor: String,
	pub buttonPressedTextColor: String,
	pub disabledColor: String,
	pub highlightColor: String,
	pub mouseDownColor: String
}

#[derive(Clone, Serialize)]
pub(super) struct DeviceSizeInfo {
	pub rows: u8,
	pub columns: u8
}

#[derive(Clone, Serialize)]
pub(super) struct DeviceInfo {
	pub id: String,
	pub name: String,
	pub size: DeviceSizeInfo,
	pub r#type: u8
}

impl DeviceInfo {
	pub(super) fn new(device: &crate::devices::DeviceInfo) -> DeviceInfo {
		DeviceInfo {
			id: device.id.clone(),
			name: device.name.clone(),
			size: DeviceSizeInfo {
				rows: device.rows,
				columns: device.columns
			},
			r#type: device.r#type
		}
	}
}

#[allow(non_snake_case)]
#[derive(Serialize)]
pub(super) struct Info {
	pub application: ApplicationInfo,
	pub plugin: PluginInfo,
	pub devicePixelRatio: u8,
	pub colors: ColoursInfo,
	pub devices: Vec<DeviceInfo>
}
