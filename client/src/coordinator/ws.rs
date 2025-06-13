use futures::{SinkExt, StreamExt, stream::SplitSink};
use tokio::net::TcpStream;

use async_tungstenite::{
    WebSocketStream,
    tokio::TokioAdapter,
    tungstenite::{Message, Utf8Bytes},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::ICoordinator;

const SERVER_URL: &str = "ws://vpn-se.pujak.ru:37080/ws";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SignalMessage {
    #[serde(rename = "type")]
    message_type: MessageType,
    data: Option<String>,
    target: Option<String>,
    room_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum MessageType {
    Join,
    Leave,
    WebRTCSignal,
}

pub(crate) struct WsCooordinator {
    socket_tx: Option<SplitSink<WebSocketStream<TokioAdapter<TcpStream>>, Message>>,
}

impl ICoordinator for WsCooordinator {
    type Message = SignalMessage;
    type Response = ();
    type CoordinationCommand = String;

    async fn send(
        &mut self,
        message: Self::Message,
    ) -> Result<Self::Response, Box<dyn std::error::Error>> {
        let text = serde_json::to_string(&message)?;
        if let Some(tx) = &mut self.socket_tx {
            tx.send(Message::Text(Utf8Bytes::from(text))).await?;
        } else {
            tracing::warn!("Null send");
        }
        Ok(())
    }

    async fn connect(
        &mut self,
        url: String,
    ) -> Result<mpsc::UnboundedReceiver<String>, Box<dyn std::error::Error>> {
        let (socket, _) = async_tungstenite::tokio::connect_async(url).await?;
        let (socket_tx, mut socket_rx) = socket.split();
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        tokio::spawn(async move {
            while let Some(Ok(message)) = socket_rx.next().await {
                if tx.send(message.to_string()).is_err() {
                    tracing::warn!("All UnbounderRecievers dropped");
                    break;
                }
            }
            tracing::warn!("Socket transimission stopped");
        });

        self.socket_tx = Some(socket_tx);

        Ok(rx)
    }
}

impl WsCooordinator {
    fn new() -> Self {
        WsCooordinator { socket_tx: None }
    }
}
