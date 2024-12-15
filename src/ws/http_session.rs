use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::ws::http_request::HttpRequest;
use crate::ws::http_request_parser::{HttpRequestParser, ParseResult};
use crate::ws::http_header::HttpHeader;

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
    pub fn new() -> Self {
        Self {
            request: HttpRequest::default(),
            request_parser: HttpRequestParser::new(),
        }
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

        println!("New http connection: {}", remote_addr);

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

            let input = String::from_utf8(buffer[..n].to_vec())
                .map_err(|e| {
                    eprintln!("Couldn't convert utf8 to valid char, error: {}", e);
                })
                .unwrap();

            let result = self.request_parser.parse(&mut self.request, input.chars());

            match result {
                ParseResult::Ok => {
                    if is_websocket_request(&self.request.headers) {
                        println!("websocket request to handle !!!!!!!!!!!!!!");
                    } else {
                        println!("normal http request to handle !!!!!!!!!");
                    }
                    println!("***********************************************");
                    println!("{:?}", self.request);
                    println!("***********************************************");
                    self.do_response().await;
                    break;
                }
                ParseResult::Indeterminate => continue,
                ParseResult::Bad => {
                    eprintln!("Could not parse request from client: {}", remote_addr);
                    return;
                }
            }
        }
    }
}

fn is_websocket_request(headers: &Vec<HttpHeader>) -> bool {
    return headers.iter().any(|header| {
        header.name == "Upgrade" && header.value == "websocket"
    });
}
