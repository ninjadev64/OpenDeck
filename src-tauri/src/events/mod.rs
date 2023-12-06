mod inbound;
pub mod frontend;

use std::collections::HashMap;

use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use futures_util::StreamExt;

use lazy_static::lazy_static;

lazy_static! {
	static ref SOCKETS: Mutex<HashMap<String, WebSocketStream<TcpStream>>> = Mutex::new(HashMap::new());
}

/// Register a plugin to send and receive events with its WebSocket.
pub async fn register_plugin(data: inbound::RegisterEvent, stream: WebSocketStream<TcpStream>) {
	SOCKETS.lock().await.insert(data.uuid.clone(), stream);
	tokio::spawn(handle_stream(data.uuid));
}

async fn handle_stream(uuid: String) {
	while let Some(data) = SOCKETS.lock().await.get_mut(&uuid).unwrap().next().await {
		println!("{}", data.unwrap());
	}
}
