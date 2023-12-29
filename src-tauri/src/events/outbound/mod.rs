pub mod keypad;
pub mod encoder;
pub mod settings;
pub mod property_inspector;

use super::SOCKETS;

use serde::Serialize;
use futures_util::SinkExt;

#[derive(Serialize)]
struct Coordinates {
	row: u8,
	column: u8
}

async fn send_to_plugin(plugin: &str, data: impl Serialize) -> Result<(), anyhow::Error> {
	let message = tokio_tungstenite::tungstenite::Message::Text(serde_json::to_string(&data).unwrap());
	SOCKETS.lock().await.get_mut(plugin).unwrap().send(message).await?;
	Ok(())
}
