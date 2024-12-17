use std::sync::Arc;
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_request_parser::{HttpRequestParser, ParseResult};
use crate::ws::http_response::HttpResponse;
use crate::ws::http_router::HttpRouter;
use crate::ws::ws_session::WsSession;

#[derive(Clone)]
pub struct HttpSession {
    request: HttpRequest,
    response: HttpResponse,
    request_parser: HttpRequestParser,
    router: Arc<HttpRouter>,
}

impl HttpSession {
    pub fn new(router: Arc<HttpRouter>) -> Self {
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(),
            request_parser: HttpRequestParser::new(),
            router,
        }
    }

    async fn do_response(&self, mut socket: TcpStream) {
        let remote = socket.peer_addr().unwrap();
        match socket.write_all(&self.response.bytes()[..]).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "Http respond can't be sent to client: {}:{}, error: {}",
                    remote.ip().to_string(),
                    remote.port(),
                    e
                );
            }
        }
    }

    pub async fn handle_socket(&mut self, mut socket: TcpStream) {
        let remote = socket
            .peer_addr()
            .map_err(|e| {
                eprintln!("Can't get remote address, error: {}", e);
            })
            .unwrap();

        let remote_addr = format!("{}:{}", remote.ip().to_string(), remote.port());

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
                    eprintln!("Can't convert utf8 to valid char, error: {}", e);
                })
                .unwrap();

            total += n;

            match self.request_parser.parse(&mut self.request, input.chars()) {
                ParseResult::Ok => {
                    break;
                }
                ParseResult::Indeterminate => continue,
                ParseResult::Bad => {
                    eprintln!("Can't parse request from client: {}", remote_addr);
                    return;
                }
            }
        }

        if is_websocket_request(&self.request.headers) {
            tokio::spawn(async move {
                let ws_session = WsSession::new(socket).await;
                let mut ws_session = match ws_session {
                    Some(ws_session) => ws_session,
                    None => {
                        eprintln!("Can't accept websockt connection");
                        return;
                    }
                };
                ws_session.handle().await;
            });
            return;
        }

        if !cleanup_socket_data(&mut socket, total).await {
            eprintln!("Can't cleanup socket");
            return;
        }

        self.router.handle(&self.request, &mut self.response);
        self.do_response(socket).await;
    }
}

fn is_websocket_request(headers: &Vec<HttpHeader>) -> bool {
    return headers
        .iter()
        .any(|header| header.name == "Upgrade" && header.value == "websocket");
}

async fn cleanup_socket_data(socket: &mut TcpStream, n: usize) -> bool {
    let mut discard_buff = vec![0; n];
    socket.read(&mut discard_buff).await.is_ok()
}
