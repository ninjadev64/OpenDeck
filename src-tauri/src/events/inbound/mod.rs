mod misc;
mod settings;

use crate::shared::ActionContext;

use serde::Deserialize;
use log::warn;

#[derive(Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "camelCase")]
pub enum RegisterEvent {
	Register { uuid: String },
	RegisterPropertyInspector { uuid: String }
}

#[derive(Deserialize)]
pub struct ContextEvent<C = ActionContext> {
	pub context: C
}

#[derive(Deserialize)]
pub struct PayloadEvent<T> {
	pub payload: T
}

#[derive(Deserialize)]
pub struct ContextAndPayloadEvent<T, C = ActionContext> {
	pub context: C,
	pub payload: T
}

#[derive(Deserialize)]
#[serde(tag = "event")]
#[serde(rename_all = "camelCase")]
pub enum InboundEventType {
	SetSettings(ContextAndPayloadEvent<serde_json::Value>),
	GetSettings(ContextEvent),
	SetGlobalSettings(ContextAndPayloadEvent<serde_json::Value, String>),
	GetGlobalSettings(ContextEvent<String>),
	OpenUrl(PayloadEvent<misc::OpenUrlEvent>),
	LogMessage(PayloadEvent<misc::LogMessageEvent>),
	SendToPropertyInspector(ContextAndPayloadEvent<serde_json::Value>),
	SendToPlugin(ContextAndPayloadEvent<serde_json::Value>)
}

pub async fn process_incoming_message(data: tokio_tungstenite::tungstenite::Message) -> Result<(), tokio_tungstenite::tungstenite::Error> {
	if let tokio_tungstenite::tungstenite::Message::Text(text) = data {
		let decoded: InboundEventType = match serde_json::from_str(&text) {
			Ok(event) => event,
			Err(_) => return Ok(())
		};

		if let Err(error) = match decoded {
			InboundEventType::SetSettings(event) => settings::set_settings(event).await,
			InboundEventType::GetSettings(event) => settings::get_settings(event).await,
			InboundEventType::SetGlobalSettings(event) => settings::set_global_settings(event).await,
			InboundEventType::GetGlobalSettings(event) => settings::get_global_settings(event).await,
			InboundEventType::OpenUrl(event) => misc::open_url(event).await,
			InboundEventType::LogMessage(event) => misc::log_message(event).await,
			InboundEventType::SendToPropertyInspector(event) => misc::send_to_property_inspector(event).await,
			InboundEventType::SendToPlugin(_) => Ok(())
		} {
			warn!("Failed to process incoming event from plugin: {}\n\tCaused by: {}", error, error.root_cause())
		}
	}

	Ok(())
}

pub async fn process_incoming_message_pi(data: tokio_tungstenite::tungstenite::Message) -> Result<(), tokio_tungstenite::tungstenite::Error> {
	if let tokio_tungstenite::tungstenite::Message::Text(text) = data {
		let decoded: InboundEventType = match serde_json::from_str(&text) {
			Ok(event) => event,
			Err(_) => return Ok(())
		};

		if let Err(error) = match decoded {
			InboundEventType::SetSettings(event) => settings::set_settings(event).await,
			InboundEventType::GetSettings(event) => settings::get_settings(event).await,
			InboundEventType::SetGlobalSettings(event) => settings::set_global_settings(event).await,
			InboundEventType::GetGlobalSettings(event) => settings::get_global_settings(event).await,
			InboundEventType::OpenUrl(event) => misc::open_url(event).await,
			InboundEventType::LogMessage(event) => misc::log_message(event).await,
			InboundEventType::SendToPlugin(event) => misc::send_to_plugin(event).await,
			_ => Ok(())
		} {
			warn!("Failed to process incoming event from property inspector: {}\n\tCaused by: {}", error, error.root_cause())
		}
	}

	Ok(())
}
