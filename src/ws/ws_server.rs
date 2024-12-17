use std::sync::Arc;
use tokio::net::TcpListener;

use crate::ws::http_router::HttpRouter;
use crate::ws::http_session;

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
                eprintln!("Could not bind to address: {}, err: {}", add, e);
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
                    eprintln!("Could not accept new Tcp connection, error: {}", e);
                    continue;
                }
            };

            let router_copy = self.router.clone();
            tokio::spawn(async move {
                let mut http_session = http_session::HttpSession::new(router_copy);
                http_session.handle_socket(socket).await;
            });
        }
    }
}
