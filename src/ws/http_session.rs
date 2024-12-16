use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};
use std::sync::Arc;

use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_response::HttpResponse;
use crate::ws::http_request_parser::{HttpRequestParser, ParseResult};
use crate::ws::ws_session::WsSession;
use crate::ws::http_router::HttpRouter;

#[derive(Clone)]
pub struct HttpSession {
    request: HttpRequest,
    response: HttpResponse,
    request_parser: HttpRequestParser,
    router: Arc<HttpRouter>,
}

impl Drop for HttpSession {
    fn drop(&mut self) {
        println!("Http session closed.");
    }
}

impl HttpSession {
    pub fn new(router: Arc<HttpRouter>) -> Self {
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(),
            request_parser: HttpRequestParser::new(),
            router
        }
    }

    async fn do_response(&self, mut socket: TcpStream) {
        let remote = socket.peer_addr().unwrap();
        match socket.write_all(&self.response.bytes()[..]).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Respond was not send to client: {}:{}, error: {}", remote.ip().to_string(), remote.port(), e);
            }
        }
    }

    pub async fn handle_socket(&mut self, mut socket: TcpStream) {
        let remote = socket
            .peer_addr()
            .map_err(|e| {
                eprintln!("Could not get remote address, error: {}", e);
            })
            .unwrap();

        let remote_addr = format!("{}:{}", remote.ip().to_string(), remote.port());

        println!("New http connection: {}", remote_addr);

        let mut buffer = [0; 1024];

        let mut total = 0;

        loop {
            let n = match socket.peek(&mut buffer).await {
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

            let input = String::from_utf8(buffer[..n].to_vec())
                .map_err(|e| {
                    eprintln!("Couldn't convert utf8 to valid char, error: {}", e);
                })
                .unwrap();

            let result = self.request_parser.parse(&mut self.request, input.chars());

            total += n;

            match result {
                ParseResult::Ok => {
                    break;
                }
                ParseResult::Indeterminate => continue,
                ParseResult::Bad => {
                    eprintln!("Could not parse request from client: {}", remote_addr);
                    return;
                }
            }
        }
        println!("***********************************************");
        println!("{:?}", self.request);
        println!("***********************************************");
        if is_websocket_request(&self.request.headers) {
            tokio::spawn(async move {
                let ws_session = WsSession::new(socket).await;
                let mut ws_session = match ws_session {
                    Some(ws_session) => ws_session,
                    None => {
                        eprintln!("Could not accept websockt connection");
                        return;
                    }
                };
                println!("Handle new websock connection");
                ws_session.handle().await;
            });
            return;
        } else {
            // cleanup socket
            let mut tmp_buff = vec![0; total];
            match socket.read_exact(&mut tmp_buff).await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Could not remove data from socket, error: {}", e);
                    return;
                }
            }
            self.router.handle(&self.request, &mut self.response);

            self.do_response(socket).await;
        }
    }
}

fn is_websocket_request(headers: &Vec<HttpHeader>) -> bool {
    return headers
        .iter()
        .any(|header| header.name == "Upgrade" && header.value == "websocket");
}
