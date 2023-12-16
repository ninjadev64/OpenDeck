pub mod misc;

use serde::Deserialize;
use tauri::{api::shell::open, Manager};

#[derive(Deserialize)]
#[serde(tag = "event", content = "payload")]
#[serde(rename_all = "camelCase")]
pub enum InboundEvent {
	OpenUrl(misc::OpenUrlEvent)
}

async fn open_url(event: misc::OpenUrlEvent) -> Result<(), anyhow::Error> {
	let app_handle = crate::APP_HANDLE.lock().await;
	open(&app_handle.as_ref().unwrap().shell_scope(), event.url, None)?;
	Ok(())
}

pub async fn process_incoming_message(data: tokio_tungstenite::tungstenite::Message) -> Result<(), tokio_tungstenite::tungstenite::Error> {
	if let tokio_tungstenite::tungstenite::Message::Text(text) = data {
		let decoded: InboundEvent = match serde_json::from_str(&text) {
			Ok(event) => event,
			Err(_) => return Ok(())
		};

		let _ = match decoded {
			InboundEvent::OpenUrl(event) => open_url(event).await
		};
	}

	Ok(())
}
