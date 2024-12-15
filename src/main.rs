mod ws;

use ws::ws_server::WsServer;

// use tokio::net::{TcpListener, TcpStream};
// use tokio_tungstenite::accept_async;
// use tokio_tungstenite::tungstenite::protocol::Message;
// use futures_util::{StreamExt, SinkExt};

// async fn handle_connection(tcp_stream: TcpStream) {
//     let remote = tcp_stream.peer_addr().map_err(|e| {
//         eprintln!("Could not get remote address, error: {}", e);
//     }).unwrap();
//     println!("New connection {}:{}", remote.ip().to_string(), remote.port());
//
//     let ws_stream = accept_async(tcp_stream).await.map_err(|e| {
//         eprintln!("Could not perform websocket connection, error: {}", e);
//     }).unwrap();
//
//     let (mut write, mut read) = ws_stream.split();
//
//     while let Some(Ok(msg)) = read.next().await {
//         if let Message::Text(text) = msg {
//             write.send(Message::Text(format!("Echo: {}", text)))
//                 .await
//                 .expect("Failed to send message");
//         }
//     }
// }

#[tokio::main]
async fn main() {
    WsServer::new().start("localhost:6969").await;
}
