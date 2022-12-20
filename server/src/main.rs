use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tracing::debug;
use tracing::info;
use tracing::warn;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
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
use sync::SyncMessage;
use tokio::sync::{mpsc, Mutex};

#[derive(clap::Parser)]
struct ServerOptions {
    #[clap(long, short, default_value = "3000")]
    port: u16,
    #[clap(long, short, default_value = "web/dist")]
    serve_dir: PathBuf,
    #[clap(long, default_value = "documents")]
    documents_dir: PathBuf,
}

struct Server {
    doc: automerge_persistent::PersistentAutomerge<automerge_persistent_fs::FsPersister>,
}

#[derive(Debug, Clone)]
struct ConnectionMetadata {
    peer_id: uuid::Uuid,
}

#[tokio::main]
async fn main() {
    let options = ServerOptions::parse();

    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/sync", get(sync_handler))
        .merge(SpaRouter::new("/", options.serve_dir).index_file("index.html"))
        .with_state(Arc::new(Mutex::new(Server {
            doc: automerge_persistent::PersistentAutomerge::load(
                automerge_persistent_fs::FsPersister::new(options.documents_dir, "test").unwrap(),
            )
            .unwrap(),
        })));

    let addr = SocketAddr::from(([127, 0, 0, 1], options.port));
    info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sync_handler(ws: WebSocketUpgrade, State(server): State<Arc<Mutex<Server>>>) -> Response {
    ws.on_upgrade(|socket| handle_sync_socket(socket, server))
}

async fn handle_sync_socket(socket: WebSocket, server: Arc<Mutex<Server>>) {
    let (sender, receiver) = socket.split();
    let connection_metadata = ConnectionMetadata {
        peer_id: uuid::Uuid::new_v4(),
    };
    info!(?connection_metadata, "New sync connection");

    let (changed_sender, changed_receiver) = tokio::sync::mpsc::channel(1);

    tokio::spawn(sync_read(
        server.clone(),
        connection_metadata.clone(),
        changed_sender,
        receiver,
    ));
    tokio::spawn(sync_write(
        server,
        connection_metadata,
        changed_receiver,
        sender,
    ));
}

#[tracing::instrument(skip(server, changed_sender, receiver))]
async fn sync_read(
    server: Arc<Mutex<Server>>,
    connection_metadata: ConnectionMetadata,
    changed_sender: mpsc::Sender<()>,
    mut receiver: SplitStream<WebSocket>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(_) => {}
                    Message::Binary(b) => {
                        // parse the sync message
                        debug!("received message");
                        let msg = SyncMessage::try_from(&b).unwrap();
                        match msg {
                            SyncMessage::Message(bytes) => {
                                debug!("parsed message as sync message, applying");
                                let msg = automerge::sync::Message::decode(&bytes).unwrap();
                                // apply the message to the document
                                server
                                    .lock()
                                    .await
                                    .doc
                                    .receive_sync_message(
                                        connection_metadata.peer_id.as_bytes().to_vec(),
                                        msg,
                                    )
                                    .unwrap();
                                let num_changes = server
                                    .lock()
                                    .await
                                    .doc
                                    .document()
                                    .get_changes(&[])
                                    .unwrap()
                                    .len();
                                let _ = changed_sender.send(()).await;
                                debug!("applied sync message, now have {}", num_changes);
                                server.lock().await.doc.flush().unwrap();
                                debug!("flushed");
                            }
                        }
                    }
                    Message::Ping(_) => {}
                    Message::Pong(_) => {}
                    Message::Close(_) => break,
                }
            }
            Err(err) => {
                warn!("failed to receive message: {}", err);
            }
        }
    }
}

#[tracing::instrument(skip(server, changed_receiver, sender))]
async fn sync_write(
    server: Arc<Mutex<Server>>,
    connection_metadata: ConnectionMetadata,
    mut changed_receiver: mpsc::Receiver<()>,
    mut sender: SplitSink<WebSocket, Message>,
) {
    while let Some(()) = changed_receiver.recv().await {
        debug!("got msg");
        if let Ok(Some(msg)) = server
            .lock()
            .await
            .doc
            .generate_sync_message(connection_metadata.peer_id.as_bytes().to_vec())
        {
            debug!("generated sync message");
            let msg = SyncMessage::Message(msg.encode());

            match Vec::try_from(msg) {
                Ok(bytes) => {
                    sender.send(Message::Binary(bytes)).await.unwrap();
                    debug!("sent sync message");
                }
                Err(err) => {
                    warn!("failed to convert sync message to bytes {}", err);
                }
            }
        }
    }
}
