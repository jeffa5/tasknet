use std::{collections::HashMap, sync::Arc};

use crate::{auth::google::Google, auth::UserSessionData, config::ServerConfig};
use async_session::MemoryStore;
use automerge_persistent_fs::FsPersisterError;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use sync::SyncMessage;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
struct ConnectionMetadata {
    peer_id: uuid::Uuid,
}

type Document = automerge_persistent::PersistentAutomerge<automerge_persistent_fs::FsPersister>;

pub struct Server {
    pub(crate) documents: HashMap<String, Document>,
    pub(crate) changed: tokio::sync::broadcast::Sender<()>,
    pub(crate) config: ServerConfig,
    pub(crate) google: Option<Google>,
    pub(crate) sessions: MemoryStore,
}

impl Server {
    fn load_document(
        &mut self,
        id: &str,
    ) -> Result<&mut Document, automerge_persistent::Error<FsPersisterError>> {
        if !self.documents.contains_key(id) {
            debug!(id, "Loading document");
            let persister =
                automerge_persistent_fs::FsPersister::new(&self.config.documents_dir, id)
                    .map_err(automerge_persistent::Error::PersisterError)?;

            let doc = automerge_persistent::PersistentAutomerge::load(persister)?;

            self.documents.insert(id.to_owned(), doc);
            debug!(id, "Loaded document");
        } else {
            debug!(id, "Document already loaded");
        }

        Ok(self.documents.get_mut(id).unwrap())
    }
}

pub async fn sync_handler(
    ws: WebSocketUpgrade,
    user: UserSessionData,
    State(server): State<Arc<Mutex<Server>>>,
) -> Response {
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
        user.clone(),
        receiver,
    ));
    tokio::spawn(sync_write(server, connection_metadata, user, sender));
}

#[tracing::instrument(skip(server, receiver))]
async fn sync_read(
    server: Arc<Mutex<Server>>,
    connection_metadata: ConnectionMetadata,
    user: UserSessionData,
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
                                    match server.load_document(user.doc_id()) {
                                        Ok(document) => {
                                            document
                                                .receive_sync_message(
                                                    connection_metadata.peer_id.as_bytes().to_vec(),
                                                    msg,
                                                )
                                                .unwrap();
                                            let num_changes =
                                                document.document().get_changes(&[]).unwrap().len();
                                            debug!(
                                                "applied sync message, now have {}",
                                                num_changes
                                            );
                                            document.flush().unwrap();
                                            debug!("flushed");
                                        }
                                        Err(err) => {
                                            warn!(id=user.doc_id(), %err, "Failed to load document, closing connection");
                                            break;
                                        }
                                    }
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
    user: UserSessionData,
    mut sender: SplitSink<WebSocket, Message>,
) {
    debug!("trying to generate initial sync message");
    {
        let mut server = server.lock().await;
        match server.load_document(user.doc_id()) {
            Ok(document) => {
                if let Ok(Some(msg)) =
                    document.generate_sync_message(connection_metadata.peer_id.as_bytes().to_vec())
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
            }
            Err(err) => {
                warn!(id=user.doc_id(), %err, "Failed to load document");
                return;
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
        match server.load_document(user.doc_id()) {
            Ok(document) => {
                if let Ok(Some(msg)) =
                    document.generate_sync_message(connection_metadata.peer_id.as_bytes().to_vec())
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
                    document.flush().unwrap();
                    debug!("flushed");
                }
            }
            Err(err) => {
                warn!(id=user.doc_id(), %err, "Failed to load document");
                return;
            }
        }
    }
}
