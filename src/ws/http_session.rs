use std::sync::Arc;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::ws::http_request::HttpRequest;
use crate::ws::http_request_parser::{HttpRequestParser, ParseResult};

#[derive(Clone)]
pub struct HttpSession {
    request: HttpRequest,
    request_parser: HttpRequestParser,
}

impl Drop for HttpSession {
    fn drop(&mut self) {
        println!("Http session closed.");
    }
}

impl HttpSession {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            request: HttpRequest::default(),
            request_parser: HttpRequestParser::new(),
        })
    }

    async fn do_response(&self) {
        todo!("To be implemented");
    }

    pub async fn handle_socket(&mut self, mut socket: TcpStream) {
        let remote = socket
            .peer_addr()
            .map_err(|e| {
                eprintln!("Could not get remote address, error: {}", e);
            })
            .unwrap();

        let remote_addr = format!("{}:{}", remote.ip().to_string(), remote.port());

        println!("New http session for client: {}", remote_addr);

        let mut buffer = [0; 1024];

        loop {
            let n = match socket.read(&mut buffer).await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!(
                        "Error during read from client: {}, error: {}",
                        remote_addr, e
                    );
                    return;
                }
            };

            if n == 0 {
                eprintln!("Can't read andy data from client: {}", remote_addr);
                break;
            }

            let result = self
                .request_parser
                .parse(&mut self.request, buffer[..n].iter().cloned());

            match result {
                ParseResult::Ok => {
                    self.do_response().await;
                    break;
                }
                ParseResult::Indeterminate => continue,
                ParseResult::Bad => {
                    eprintln!();
                    return;
                }
            }
        }
    }
}
