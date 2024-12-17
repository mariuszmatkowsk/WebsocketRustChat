mod ws;

use std::env;
use std::path::Path;
use std::sync::Arc;

use ws::file_storage::FileStorage;
use ws::http_header::HttpHeader;
use ws::http_request::HttpRequest;
use ws::http_response::{HttpResponse, StatusType};
use ws::http_router::HttpRouter;
use ws::method::Method;
use ws::ws_server::WsServer;

fn extension_to_mimo_type(extension: &str) -> String {
    match extension {
        ".html" => String::from("text/html"),
        ".png" => String::from("image/png"),
        _ => {
            todo!()
        }
    }
}

pub struct StaticFileHandler {
    file_storage: Arc<FileStorage>,
    file_name: String,
}

impl StaticFileHandler {
    pub fn handle(&self, _request: &HttpRequest, response: &mut HttpResponse) {
        let file_content = match self.file_storage.get(&self.file_name) {
            Some(file_content) => file_content,
            None => {
                todo!("Response Not found");
            }
        };

        let mut headers = Vec::new();
        headers.push(HttpHeader::new(
            String::from("Content-Type"),
            extension_to_mimo_type(&self.file_name[self.file_name.find('.').unwrap()..]),
        ));

        *response = HttpResponse::new(StatusType::Ok, headers, file_content.to_vec());
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: WebsocketRustChat <doc_root>");
        std::process::exit(1);
    }
    let doc_root_path = Path::new(&args[1]);
    let file_storage = match FileStorage::new(doc_root_path) {
        Some(file_storage) => file_storage,
        None => {
            eprintln!("Could not load files from provided directory");
            std::process::exit(1);
        }
    };

    let file_storage = Arc::new(file_storage);

    let mut http_router = HttpRouter::new(file_storage.clone());
    http_router
        .add_route(Method::Get, String::from("/"), {
            let file_storage = file_storage.clone();
            move |req, resp| {
                StaticFileHandler {
                    file_storage: file_storage.clone(),
                    file_name: String::from("index.html"),
                }
                .handle(req, resp);
            }
        })
        .add_route(Method::Get, String::from("/index.html"), {
            let file_storage = file_storage.clone();
            move |req, resp| {
                StaticFileHandler {
                    file_storage: file_storage.clone(),
                    file_name: String::from("index.html"),
                }
                .handle(req, resp);
            }
        })
        .add_route(Method::Get, String::from("/favicon.ico"), {
            let file_storage = file_storage.clone();
            move |req, resp| {
                StaticFileHandler {
                    file_storage: file_storage.clone(),
                    file_name: String::from("favicon.png"),
                }
                .handle(req, resp);
            }
        });

    WsServer::new(http_router).start("localhost:6969").await;
}
