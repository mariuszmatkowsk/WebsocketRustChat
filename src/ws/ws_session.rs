use crate::ws::ws_message::MessageType;
use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc::channel, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

const USAGE_MSG: &str = "To use chat, you need to set your nickname.
Usage:
    /nick <nickname>                - set your nickname
    /quit                           - leave chat
    /help                           - show help
    /private <nickname> <message>   - send private message";

type SocketReadHalf<S> = SplitStream<WebSocketStream<S>>;
type SocketWriteHalf<S> = Arc<Mutex<SplitSink<WebSocketStream<S>, Message>>>;
pub type Clients<S> = Arc<Mutex<HashMap<String, SocketWriteHalf<S>>>>;

pub struct WsSession<S> {
    socket_read_half: SocketReadHalf<S>,
    socket_write_half: SocketWriteHalf<S>,
    clients: Clients<S>,
    nickname: Arc<Mutex<Option<String>>>,
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
        write_half
            .lock()
            .await
            .send(Message::Text(USAGE_MSG.to_string()))
            .await
            .expect("Should send usage message");

        Some(Self {
            socket_read_half: read_half,
            socket_write_half: write_half,
            clients,
            nickname: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn handle_ws_connection(&mut self) {
        let (tx, mut rx) = channel::<String>(5);
        loop {
            let clients_clone = Arc::clone(&self.clients);
            let client_nickname = Arc::clone(&self.nickname);

            tokio::select! {
                message = rx.recv() => {
                    if let Some(message) = message {
                        if let Some(ref nick) = *client_nickname.lock().await {
                            Self::brodcast_message(
                                nick,
                                message.clone(),
                                Arc::clone(&clients_clone),
                            )
                            .await;
                        }
                    }
                }
                message = self.socket_read_half.next() => {
                    let msg = match message {
                        Some(Ok(Message::Text(msg))) => msg,
                        Some(Ok(Message::Binary(data))) => String::from_utf8(data).unwrap(),
                        Some(Ok(Message::Close(_))) => {
                            self.handle_close_message().await;
                            return;
                        }
                        _ => {
                            // Do nothing for Ping/Pong message
                            continue;
                        }
                    };

                    if let Some(m) = self.parse_command(msg) {
                        match m {
                            MessageType::Nick(nick_message) => {
                                if self.nickname.lock().await.is_none() {
                                    *self.nickname.lock().await = Some(nick_message.nick.clone());
                                    self.clients.lock().await.insert(
                                        nick_message.nick.clone(),
                                        Arc::clone(&self.socket_write_half),
                                    );
                                    self.send_to_self(format!(
                                        "Hello {}, now you can send messages",
                                        nick_message.nick
                                    ));
                                    continue;
                                }
                            }
                            MessageType::Chat(chat_message) => match *self.nickname.lock().await {
                                None => {
                                    self.send_to_self(String::from(
                                        "Please enter your nickname: /nick <your_nickname>",
                                    ));
                                }
                                Some(_) => match tx.send(chat_message.message).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        eprintln!("Could not brodcast message, error: {}", e);
                                    }
                                },
                            },
                            MessageType::Private(private_message) => {
                                let clients = self.clients.lock().await;
                                if let Some(c) = clients.get(&private_message.receiver) {
                                    c.lock()
                                        .await
                                        .send(Message::Text(private_message.message))
                                        .await
                                        .expect("Private message to be sent.");
                                } else {
                                    self.send_to_self(format!(
                                        "Client with nickname: {} is not connected to chat",
                                        private_message.receiver
                                    ));
                                }
                            }
                            MessageType::Quit(_) => {
                                let mut nick = self.nickname.lock().await;

                                match *nick {
                                    Some(ref n) => {
                                        self.send_to_self(format!("You left the chat."));
                                        self.clients.lock().await.remove(n);
                                        *nick = None;
                                    }
                                    None => {
                                        self.send_to_self(format!(
                                            "Leave impossible, you are not in the chat"
                                        ));
                                    }
                                }
                            }
                            MessageType::Help(_) => {
                                self.send_to_self(USAGE_MSG.to_string());
                            }
                        }
                    } else {
                        self.send_to_self(format!("Command not supported."));
                    }
                }
            }
        }
    }

    async fn brodcast_message(sender_nick: &str, message: String, clients: Clients<S>) {
        for (nick, client) in clients.lock().await.iter() {
            if nick == sender_nick {
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

    async fn handle_close_message(&self) {
        match *self.nickname.lock().await {
            Some(ref nick) => {
                self.clients.lock().await.remove(nick);
            }
            None => {}
        }
        println!("Client disconnected");
    }

    fn send_to_self(&self, message: String) {
        let socket_write_half_copy = Arc::clone(&self.socket_write_half);
        tokio::spawn(async move {
            socket_write_half_copy
                .lock()
                .await
                .send(Message::text(format!("{}", message)))
                .await
                .expect("Message should be sent to self.");
        });
    }

    fn parse_command(&self, input: String) -> Option<MessageType> {
        let message = serde_json::from_str::<MessageType>(&input);
        match message {
            Ok(message) => Some(message),
            Err(_) => None,
        }
    }
}
