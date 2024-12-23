use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::ws::http_router::HttpRouter;
use crate::ws::http_session::{HttpHandleError, HttpSession};
use crate::ws::ws_session::WsSession;

pub struct WsServer {
    router: Arc<HttpRouter>,
}

impl WsServer {
    pub fn new(router: HttpRouter) -> Self {
        Self {
            router: Arc::new(router),
        }
    }

    pub async fn start(self, add: &str) {
        let tcp_listener = TcpListener::bind(add)
            .await
            .map_err(|e| {
                eprintln!("Could not bind to address: {}, error: {}", add, e);
            })
            .unwrap();

        let clients = Arc::new(Mutex::new(HashMap::new()));

        loop {
            let mut socket = match tcp_listener.accept().await {
                Ok((socket, remote_add)) => {
                    println!(
                        "Handling new connection: {}:{}",
                        remote_add.ip().to_string(),
                        remote_add.port()
                    );
                    socket
                }
                Err(e) => {
                    eprintln!("Could not accept new Tcp connection, error: {}", e);
                    continue;
                }
            };

            let router_copy = Arc::clone(&self.router);
            let clients_copy = Arc::clone(&clients);
            tokio::spawn(async move {
                let mut http_session = HttpSession::new(router_copy);
                if let Err(error) = http_session.handle_socket(&mut socket).await {
                    if error == HttpHandleError::WebsocketProtocol {
                        let ws_session = WsSession::new(socket, clients_copy).await;
                        let mut ws_session = match ws_session {
                            Some(ws_session) => ws_session,
                            None => {
                                eprintln!("Can't accept websockt connection");
                                return;
                            }
                        };
                        ws_session.handle_ws_connection().await;
                    }
                }
            });
        }
    }
}
