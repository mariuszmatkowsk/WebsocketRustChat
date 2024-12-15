use crate::ws::http_session;

use tokio::net::TcpListener;

pub struct WsServer {}

impl WsServer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn start(self, add: &str) {
        let tcp_listener = TcpListener::bind(add)
            .await
            .map_err(|e| {
                eprintln!("Culd not bind to address: {}, err: {}", add, e);
            })
            .unwrap();

        loop {
            let socket = match tcp_listener.accept().await {
                Ok((socket, remote_add)) => {
                    println!(
                        "Handling new connection: {}:{}",
                        remote_add.ip().to_string(),
                        remote_add.port()
                    );
                    socket
                }
                Err(e) => {
                    eprintln!("Could not accept new connection, error: {}", e);
                    continue;
                }
            };
            
            tokio::spawn(async move {
                let mut http_session = http_session::HttpSession::new();
                http_session.handle_socket(socket).await;
            });
        }
    }
}
