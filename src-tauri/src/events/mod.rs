mod inbound;
pub mod outbound;
pub mod frontend;

use inbound::RegisterEvent;

use std::collections::HashMap;

use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{stream::SplitSink, StreamExt, TryStreamExt};

use lazy_static::lazy_static;

lazy_static! {
	static ref SOCKETS: Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>> = Mutex::new(HashMap::new());
	static ref PROPERTY_INSPECTOR_SOCKETS: Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>> = Mutex::new(HashMap::new());
}

/// Register a plugin or property inspector to send and receive events with its WebSocket.
pub async fn register_plugin(event: RegisterEvent, stream: WebSocketStream<TcpStream>) {
	let (read, write) = stream.split();
	match event {
		RegisterEvent::Register { uuid } => {
			SOCKETS.lock().await.insert(uuid, read);
			tokio::spawn(write.try_for_each(inbound::process_incoming_message));
		}
		RegisterEvent::RegisterPropertyInspector { uuid } => {
			PROPERTY_INSPECTOR_SOCKETS.lock().await.insert(uuid, read);
			tokio::spawn(write.try_for_each(inbound::process_incoming_message_pi));
		}
	};
}
