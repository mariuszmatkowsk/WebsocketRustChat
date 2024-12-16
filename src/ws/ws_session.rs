use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{accept_async, WebSocketStream};

pub struct WsSession<S> {
    ws_socket: WebSocketStream<S>,
}

impl<S> WsSession<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new(socket: S) -> Option<Self> {
        let ws_socket = accept_async(socket).await.map_err(|e| {
            eprintln!("Could not accept websocket, error {}", e);
            return ()
        }).unwrap();
        println!("New websocket connection established.");
        Some(Self {ws_socket})
    }

    pub async fn handle(&mut self) {
        while let Some(Ok(msg)) = self.ws_socket.next().await {
            if let Message::Text(text) = msg {
                self.ws_socket.send(Message::Text(format!("Echo: {}", text)))
                    .await
                    .expect("Failed to send message");
            }
        }

    }
}
