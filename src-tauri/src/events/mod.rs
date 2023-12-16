mod inbound;
pub mod outbound;
pub mod frontend;

use std::collections::HashMap;

use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use futures_util::{stream::SplitSink, StreamExt, TryStreamExt};

use lazy_static::lazy_static;

lazy_static! {
	static ref SOCKETS: Mutex<HashMap<String, SplitSink<WebSocketStream<TcpStream>, Message>>> = Mutex::new(HashMap::new());
}

/// Register a plugin to send and receive events with its WebSocket.
pub async fn register_plugin(data: inbound::RegisterEvent, stream: WebSocketStream<TcpStream>) {
	let (read, write) = stream.split();
	tokio::spawn(write.try_for_each(inbound::process_incoming_message));
	SOCKETS.lock().await.insert(data.uuid.clone(), read);
}
