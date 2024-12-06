use super::PayloadEvent;

use serde::Deserialize;

pub async fn register_device(uuid: &str, mut event: PayloadEvent<crate::devices::DeviceInfo>) -> Result<(), anyhow::Error> {
	if Some(uuid) == crate::devices::DEVICE_NAMESPACES.read().await.get(&event.payload.id[..2]).map(|x| x.as_str()) {
		event.payload.plugin = uuid.to_owned();
		crate::devices::register_device(event.payload).await;
		Ok(())
	} else {
		Err(anyhow::anyhow!("plugin {uuid} is not registered for device namespace {}", &event.payload.id[..2]))
	}
}

#[derive(Deserialize)]
pub struct PressPayload {
	pub device: String,
	pub position: u8,
}

pub async fn key_down(event: PayloadEvent<PressPayload>) -> Result<(), anyhow::Error> {
	crate::events::outbound::keypad::key_down(&event.payload.device, event.payload.position).await
}

pub async fn key_up(event: PayloadEvent<PressPayload>) -> Result<(), anyhow::Error> {
	crate::events::outbound::keypad::key_up(&event.payload.device, event.payload.position).await
}

#[derive(Deserialize)]
pub struct TicksPayload {
	pub device: String,
	pub position: u8,
	pub ticks: i16,
}

pub async fn encoder_change(event: PayloadEvent<TicksPayload>) -> Result<(), anyhow::Error> {
	crate::events::outbound::encoder::dial_rotate(&event.payload.device, event.payload.position, event.payload.ticks).await
}

pub async fn encoder_down(event: PayloadEvent<PressPayload>) -> Result<(), anyhow::Error> {
	crate::events::outbound::encoder::dial_press(&event.payload.device, "dialDown", event.payload.position).await
}

pub async fn encoder_up(event: PayloadEvent<PressPayload>) -> Result<(), anyhow::Error> {
	crate::events::outbound::encoder::dial_press(&event.payload.device, "dialUp", event.payload.position).await
}
