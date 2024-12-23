use futures::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc::channel, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message, WebSocketStream};

enum CommandType {
    EnterNickname(String),
    PrivateMessage(String, String),
    Leave,
    MessageToAll(String),
    ShowHelp,
    Unknown(String),
}

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
    nickname: Option<String>,
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
            nickname: None,
        })
    }

    pub async fn handle_ws_connection(&mut self) {
        let (tx, mut rx) = channel::<String>(5);

        let clients_clone = self.clients.clone();
        let client_nickname = self.nickname.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Some(message) => {
                        if let Some(nick) = &client_nickname {
                            Self::brodcast_message(nick, message.clone(), clients_clone.clone())
                                .await;
                        };
                    }
                    None => {}
                }
            }
        });

        loop {
            let msg = match self.socket_read_half.next().await {
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

            // Parse message
            match self.parse_command(msg) {
                CommandType::EnterNickname(nick) => {
                    if self.nickname.is_none() {
                        self.nickname = Some(nick.clone());
                        self.send_to_self(format!("Hello {}, now you can send messages", nick));
                        continue;
                    }
                }
                CommandType::MessageToAll(message) => match self.nickname {
                    None => {
                        self.send_to_self(String::from(
                            "Please enter your nickname: /nick <your_nickname>",
                        ));
                    }
                    Some(_) => match tx.send(message).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Could not brodcast message, error: {}", e);
                        }
                    },
                },
                CommandType::PrivateMessage(_client, _message) => {
                    todo!("Implement send private message, to specified client");
                }
                CommandType::Leave => {
                    match &self.nickname {
                        Some(nick) => {
                            self.send_to_self(format!("You left the chat."));
                            self.clients.lock().await.remove(nick);
                            self.nickname = None;
                        }
                        None => {
                            self.send_to_self(format!("Leave impossible, you are not in the chat"));
                        }
                    }
                }
                CommandType::ShowHelp => {
                    self.send_to_self(USAGE_MSG.to_string());
                }
                CommandType::Unknown(command) => {
                    self.send_to_self(format!("Command: {} not supported.", command));
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
        match &self.nickname {
            Some(nick) => {
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

    fn parse_command(&self, input: String) -> CommandType {
        match input.strip_prefix('/') {
            None => CommandType::MessageToAll(input),
            // handle command
            Some(after_prefix) => {
                match after_prefix {
                    "help" => return CommandType::ShowHelp,
                    "quit" => return CommandType::Leave,
                    _ => {}
                }

                if let Some((command, rest)) = after_prefix.split_once(' ') {
                    match command {
                        "nick" => return CommandType::EnterNickname(rest.to_string()),
                        "private" => {
                            if let Some((nick, message)) = rest.split_once(' ') {
                                if !nick.is_empty() {
                                    return CommandType::PrivateMessage(
                                        nick.to_string(),
                                        message.to_string(),
                                    );
                                };
                            }
                        }
                        _ => return CommandType::Unknown(command.to_string()),
                    };
                }
                return CommandType::Unknown(after_prefix.to_string());
            }
        }
    }
}
