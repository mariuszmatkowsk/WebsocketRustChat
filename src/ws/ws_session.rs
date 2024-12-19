use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc::channel, Mutex};
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
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
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
        let (tx, mut rx) = channel::<String>(5);

        let clients_clone = self.clients.clone();
        let socket_write_half = self.socket_write_half.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    None => (),
                    Some(message) => {
                        Self::brodcast_message(
                            socket_write_half.clone(),
                            message.clone(),
                            clients_clone.clone(),
                        )
                        .await;
                    }
                }
            }
        });

        loop {
            let msg = match self.socket_read_half.next().await {
                Some(Ok(Message::Text(msg))) => msg,
                Some(Ok(Message::Binary(data))) => String::from_utf8(data).unwrap(),
                Some(Ok(Message::Close(_))) => {
                    let mut clients = self.clients.lock().await;
                    if let Some(pos) = clients
                        .iter()
                        .position(|c| Arc::ptr_eq(c, &self.socket_write_half))
                    {
                        clients.remove(pos);
                        eprintln!("Client disconnected");
                    }
                    return;
                }
                _ => {
                    todo!("Not handled case");
                }
            };

            match tx.send(msg).await {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Could not send message to brodkcast task, error: {}", e);
                }
            }
        }
    }

    async fn brodcast_message(sender: SocketWriteHalf<S>, message: String, clients: Clients<S>) {
        for client in clients.lock().await.iter() {
            if Arc::ptr_eq(client, &sender) {
                continue;
            }

            let message_clone = message.clone();
            let client_clone = client.clone();
            tokio::spawn(async move {
                client_clone
                    .lock()
                    .await
                    .send(Message::text(format!("{}", message_clone)))
                    .await
                    .expect("Message should be sent");
            });
        }
    }
}
