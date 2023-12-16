use super::PayloadEvent;

use tauri::{api::shell::open, Manager};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct OpenUrlEvent {
	pub url: String
}

pub async fn open_url(event: PayloadEvent<OpenUrlEvent>) -> Result<(), anyhow::Error> {
	let app_handle = crate::APP_HANDLE.lock().await;
	open(&app_handle.as_ref().unwrap().shell_scope(), event.payload.url, None)?;
	Ok(())
}
