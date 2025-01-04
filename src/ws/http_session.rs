use std::sync::Arc;
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

use crate::ws::http_header::HttpHeader;
use crate::ws::http_request::HttpRequest;
use crate::ws::http_request_parser::{HttpRequestParser, ParseResult};
use crate::ws::http_response::HttpResponse;
use crate::ws::http_router::HttpRouter;

#[derive(PartialEq, Eq)]
pub enum HttpHandleError {
    WebsocketProtocol,
    ParseRequestError,
    SocketConnectionError,
}

pub type HandleResult = std::result::Result<(), HttpHandleError>;

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

    async fn do_response(&self, socket: &mut TcpStream, remote_addr: &str) {
        match socket.write_all(&self.response.bytes()[..]).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "Http respond can't be sent to client: {}, error: {}",
                    remote_addr, e
                );
            }
        }
    }

    pub async fn handle_socket(&mut self, socket: &mut TcpStream) -> HandleResult {
        let remote_addr = match socket.peer_addr() {
            Ok(remote) => format!("{}:{}", remote.ip().to_string(), remote.port()),
            Err(e) => {
                eprintln!("Can't get remote address, error: {}", e);
                return Err(HttpHandleError::SocketConnectionError);
            }
        };

        let mut buffer = [0; 1024];
        let mut total = 0;

        while let Ok(n) = socket.peek(&mut buffer).await {
            if n == 0 {
                eprintln!("Can't read any data from client: {}", remote_addr);
                return Err(HttpHandleError::SocketConnectionError);
            }

            let input = if let Ok(input) = String::from_utf8(buffer[..n].to_vec()) {
                input
            } else {
                eprintln!("Can't convert request to valid utf8 charactre");
                return Err(HttpHandleError::ParseRequestError);
            };

            total += n;

            match self.request_parser.parse(&mut self.request, input.chars()) {
                ParseResult::Ok => {
                    break;
                }
                ParseResult::Indeterminate => continue,
                ParseResult::Bad => {
                    eprintln!("Can't parse request from client: {}", remote_addr);
                    return Err(HttpHandleError::ParseRequestError);
                }
            }
        }

        if is_websocket_request(&self.request.headers) {
            return Err(HttpHandleError::WebsocketProtocol);
        }

        if !cleanup_socket_data(socket, total).await {
            eprintln!("Can't cleanup socket");
            return Err(HttpHandleError::SocketConnectionError);
        }

        self.router.handle(&self.request, &mut self.response);
        self.do_response(socket, &remote_addr).await;
        Ok(())
    }
}

fn is_websocket_request(headers: &Vec<HttpHeader>) -> bool {
    return headers
        .iter()
        .any(|header| header.name == "Upgrade" && header.value == "websocket");
}

async fn cleanup_socket_data(socket: &mut TcpStream, n: usize) -> bool {
    let mut discard_buff = vec![0; n];
    socket.read_exact(&mut discard_buff).await.is_ok()
}
