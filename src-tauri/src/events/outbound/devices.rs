use super::send_to_all_plugins;

use crate::plugins::info_param::DeviceInfo;

use serde::Serialize;

#[derive(Serialize)]
#[allow(non_snake_case)]
struct DeviceDidConnectEvent {
	event: &'static str,
	device: String,
	deviceInfo: DeviceInfo,
}

#[derive(Serialize)]
struct DeviceDidDisconnectEvent {
	event: &'static str,
	device: String,
}

pub async fn device_did_connect(id: &str, info: DeviceInfo) -> Result<(), anyhow::Error> {
	send_to_all_plugins(&DeviceDidConnectEvent {
		event: "deviceDidConnect",
		device: id.to_owned(),
		deviceInfo: info,
	})
	.await
}

pub async fn device_did_disconnect(id: &str) -> Result<(), anyhow::Error> {
	send_to_all_plugins(&DeviceDidDisconnectEvent {
		event: "deviceDidDisconnect",
		device: id.to_owned(),
	})
	.await
}
