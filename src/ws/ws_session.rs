use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};

type SocketReadHalf<S> = SplitStream<WebSocketStream<S>>;
type SocketWriteHalf<S> = Arc<Mutex<SplitSink<WebSocketStream<S>, Message>>>;
pub type Clients<S> = Arc<Mutex<Vec<SocketWriteHalf<S>>>>;

pub struct WsSession<S> {
    socket_read_half: SocketReadHalf<S>,
    socket_write_half: SocketWriteHalf<S>,
    clients: Clients<S>,
}

impl<S> WsSession<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new(socket: S, clients: Clients<S>) -> Option<Self> {
        let ws_socket = accept_async(socket)
            .await
            .map_err(|e| {
                eprintln!("Could not accept websocket connection, error {}", e);
                return ();
            })
            .unwrap();
        let (write_half, read_half) = ws_socket.split();
        let write_half = Arc::new(Mutex::new(write_half));
        clients.lock().await.push(write_half.clone());
        Some(Self {
            socket_read_half: read_half,
            socket_write_half: write_half,
            clients,
        })
    }

    pub async fn handle(&mut self) {
        loop {
            let msg = match self.socket_read_half.next().await {
                Some(Ok(Message::Text(msg))) => msg,
                Some(Ok(Message::Binary(data))) => String::from_utf8(data).unwrap(),
                Some(Ok(Message::Close(_))) => {
                    todo!("Close message received");
                }
                _ => {
                    todo!("Not handled case");
                }
            };
            self.brodcast_message(msg).await;
        }
    }

    async fn brodcast_message(&self, message: String) {
        let clients = {
            let clients_lock = self.clients.lock().await;
            clients_lock.clone()
        };

        for write_client in clients.iter() {
            if Arc::ptr_eq(write_client, &self.socket_write_half) {
                continue;
            }
            let mut write_client = write_client.lock().await;
            write_client
                .send(Message::text(format!("{}", message)))
                .await
                .expect("Message was sent");
        }
    }
}
