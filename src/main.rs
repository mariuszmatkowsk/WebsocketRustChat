mod ws;

use std::env;
use std::path::Path;
use std::sync::Arc;

use ws::file_storage::FileStorage;
use ws::http_router::HttpRouter;
use ws::method::Method;
use ws::ws_server::WsServer;
use ws::static_file_handler::StaticFileHandler;
use ws::middleware::{Middleware, RequestLogger};

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
        .add_route(
            Method::Get,
            String::from("/"),
            Middleware::new(
                RequestLogger::new(),
                StaticFileHandler::new(file_storage.clone(), String::from("index.html"))),
        )
        .add_route(
            Method::Get,
            String::from("/index.html"),
            Middleware::new(
                RequestLogger::new(),
                StaticFileHandler::new(file_storage.clone(), String::from("index.html"))),
        )
        .add_route(
            Method::Get,
            String::from("/script.js"),
            StaticFileHandler::new(file_storage.clone(), String::from("script.js"))
        )
        .add_route(
            Method::Get,
            String::from("/favicon.ico"),
            Middleware::new(
                RequestLogger::new(),
                StaticFileHandler::new(file_storage.clone(), String::from("favicon.png")))
        );

    WsServer::new(http_router).start("localhost:6969").await;
}
