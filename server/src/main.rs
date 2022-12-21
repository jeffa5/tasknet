use async_session::MemoryStore;
use config::ServerConfig;
use google::Google;
use google::UserSessionData;
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
use tokio::sync::Mutex;

mod config;
mod google;

#[derive(clap::Parser)]
struct ServerOptions {
    #[clap(long, short, default_value = "config.yaml")]
    config_file: PathBuf,
}

pub struct Server {
    doc: automerge_persistent::PersistentAutomerge<automerge_persistent_fs::FsPersister>,
    changed: tokio::sync::broadcast::Sender<()>,
    config: ServerConfig,
    google: Option<Google>,
    sessions: MemoryStore,
}

#[derive(Debug, Clone)]
struct ConnectionMetadata {
    peer_id: uuid::Uuid,
}

#[tokio::main]
async fn main() {
    let options = ServerOptions::parse();

    let config = config::ServerConfig::load(&options.config_file);

    tracing_subscriber::fmt::init();

    let (changed, _) = tokio::sync::broadcast::channel(1);

    let port = config.port;

    let google = if let Some(config) = config.google.as_ref() {
        Some(Google::new(config).await)
    } else {
        None
    };

    let app = Router::new()
        .route("/sync", get(sync_handler))
        .route("/auth/google/sign_in", get(google::sign_in_handler))
        .route("/auth/google/sign_out", get(google::sign_out_handler))
        .route("/auth/google/callback", get(google::callback_handler))
        .merge(SpaRouter::new("/", &config.serve_dir).index_file("index.html"))
        .with_state(Arc::new(Mutex::new(Server {
            doc: automerge_persistent::PersistentAutomerge::load(
                automerge_persistent_fs::FsPersister::new(&config.documents_dir, "test").unwrap(),
            )
            .unwrap(),
            changed,
            config,
            google,
            sessions: MemoryStore::new(),
        })));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://localhost:{}", port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sync_handler(ws: WebSocketUpgrade, user: UserSessionData, State(server): State<Arc<Mutex<Server>>>) -> Response {
    ws.on_upgrade(|socket| handle_sync_socket(socket, server, user))
}

async fn handle_sync_socket(socket: WebSocket, server: Arc<Mutex<Server>>, user: UserSessionData) {
    let (sender, receiver) = socket.split();
    let connection_metadata = ConnectionMetadata {
        peer_id: uuid::Uuid::new_v4(),
    };
    info!(?connection_metadata, "New sync connection");

    tokio::spawn(sync_read(
        server.clone(),
        connection_metadata.clone(),
        receiver,
    ));
    tokio::spawn(sync_write(server, connection_metadata, sender));
}

#[tracing::instrument(skip(server, receiver))]
async fn sync_read(
    server: Arc<Mutex<Server>>,
    connection_metadata: ConnectionMetadata,
    mut receiver: SplitStream<WebSocket>,
) {
    let changed = server.lock().await.changed.clone();
    debug!("waiting for messages from client");
    while let Some(msg) = receiver.next().await {
        debug!("received msg");
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(_) => {}
                    Message::Binary(b) => {
                        // parse the sync message
                        debug!("received binary ws message");
                        let msg = SyncMessage::try_from(&b).unwrap();
                        match msg {
                            SyncMessage::Message(bytes) => {
                                {
                                    debug!("parsed message into sync message");
                                    let msg = automerge::sync::Message::decode(&bytes).unwrap();
                                    // apply the message to the document
                                    let mut server = server.lock().await;
                                    server
                                        .doc
                                        .receive_sync_message(
                                            connection_metadata.peer_id.as_bytes().to_vec(),
                                            msg,
                                        )
                                        .unwrap();
                                    let num_changes =
                                        server.doc.document().get_changes(&[]).unwrap().len();
                                    debug!("applied sync message, now have {}", num_changes);
                                    server.doc.flush().unwrap();
                                    debug!("flushed");
                                }
                                let _ = changed.send(());
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

#[tracing::instrument(skip(server, sender))]
async fn sync_write(
    server: Arc<Mutex<Server>>,
    connection_metadata: ConnectionMetadata,
    mut sender: SplitSink<WebSocket, Message>,
) {
    debug!("trying to generate initial sync message");
    if let Ok(Some(msg)) = server
        .lock()
        .await
        .doc
        .generate_sync_message(connection_metadata.peer_id.as_bytes().to_vec())
    {
        debug!("generated initial sync message");
        let msg = SyncMessage::Message(msg.encode());

        match Vec::try_from(msg) {
            Ok(bytes) => {
                sender.send(Message::Binary(bytes)).await.unwrap();
                debug!("sent initial sync message");
            }
            Err(err) => {
                warn!("failed to convert sync message to bytes {}", err);
            }
        }
    }

    let mut changed = {
        let server = server.lock().await;
        server.changed.subscribe()
    };
    debug!("waiting for changes");
    while let Ok(()) = changed.recv().await {
        debug!("notified of change");
        let mut server = server.lock().await;
        if let Ok(Some(msg)) = server
            .doc
            .generate_sync_message(connection_metadata.peer_id.as_bytes().to_vec())
        {
            debug!("generated sync message");
            let msg = SyncMessage::Message(msg.encode());

            match Vec::try_from(msg) {
                Ok(bytes) => match sender.send(Message::Binary(bytes)).await {
                    Ok(()) => debug!("sent sync message"),
                    Err(err) => {
                        warn!("failed to send sync message {}", err);
                        break;
                    }
                },
                Err(err) => {
                    warn!("failed to convert sync message to bytes {}", err);
                }
            }
            server.doc.flush().unwrap();
            debug!("flushed");
        }
    }
}
