use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use axum_extra::routing::SpaRouter;
use clap::Parser;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};

#[derive(clap::Parser)]
struct ServerOptions {
    #[clap(long, short, default_value = "3000")]
    port: u16,
    #[clap(long, short, default_value = "web/dist")]
    serve_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let options = ServerOptions::parse();

    let app = Router::new()
        .route("/sync", get(sync_handler))
        .merge(SpaRouter::new("/", options.serve_dir).index_file("index.html"));

    let addr = SocketAddr::from(([127, 0, 0, 1], options.port));
    println!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sync_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_sync_socket)
}

async fn handle_sync_socket(socket: WebSocket) {
    let (sender, receiver) = socket.split();
    tokio::spawn(sync_read(receiver));
    tokio::spawn(sync_write(sender));
}

async fn sync_read(mut receiver: SplitStream<WebSocket>) {
    while let Some(msg) = receiver.next().await {
        println!("received message: {:?}", msg);
    }
}

async fn sync_write(mut sender: SplitSink<WebSocket, Message>) {
    sender.send("hello".into()).await.unwrap();
}
