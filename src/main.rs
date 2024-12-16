mod ws;

use std::fs::read;
use ws::http_header::HttpHeader;
use ws::http_request::HttpRequest;
use ws::http_response::{HttpResponse, StatusType};
use ws::http_router::HttpRouter;
use ws::method::Method;
use ws::ws_server::WsServer;

fn handle_index(_request: &HttpRequest, response: &mut HttpResponse) {
    println!("handling index");
    let index_content =
        match read("/home/mariusz/code/rust/public_repos/WebsocketRustChat/index.html") {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Could not read content of index.html, error: {}", e);
                response.status = StatusType::InternalServerError;
                response.body =
                    b"<html><head></head><body>Internal Server Error</body></html>".to_vec();
                response.headers.push(HttpHeader::new(
                    String::from("Content-Type"),
                    String::from("text/html"),
                ));
                response.headers.push(HttpHeader::new(
                    String::from("Content-Length"),
                    response.body.len().to_string(),
                ));
                return;
            }
        };

    response.status = StatusType::Ok;
    response.body = index_content;
    response.headers.push(HttpHeader::new(
        String::from("Content-Type"),
        String::from("text/html"),
    ));
    response.headers.push(HttpHeader::new(
        String::from("Content-Length"),
        response.body.len().to_string(),
    ));
}

#[tokio::main]
async fn main() {
    let mut http_router = HttpRouter::new();

    http_router
        .add_route(Method::Get, String::from("/"), handle_index)
        .add_route(Method::Get, String::from("/index.html"), handle_index);

    WsServer::new(http_router).start("localhost:6969").await;
}
