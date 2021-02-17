use std::{
    collections::HashSet,
    convert::TryFrom,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub trait SyncedBackend {
    fn db(&self) -> automerge::Backend;

    fn db_mut(&mut self) -> automerge::Backend;

    fn get_heads(&self) -> Heads {
        self.db().get_heads()
    }

    fn get_changes(&self, heads: Heads) -> Changes {
        let db = self.db();
        let changes = db.get_changes(&heads);
        let changes = changes.iter().map(|c| c.decode()).collect();
        changes
    }

    fn apply_remote_changes(&mut self, changes: Changes) {
        let changes: Vec<_> = changes.iter().map(automerge::Change::from).collect();
        self.db().apply_changes(changes).unwrap();
    }
}

pub struct Connection;
impl Connection {
    pub async fn handle(
        send_msg: tokio::sync::mpsc::Sender<Message>,
        mut recv_msg: tokio::sync::mpsc::Receiver<Message>,
        get_heads: tokio::sync::mpsc::Sender<
            tokio::sync::oneshot::Sender<Vec<automerge_protocol::ChangeHash>>,
        >,
        get_changes: tokio::sync::mpsc::Sender<(
            Vec<automerge_protocol::ChangeHash>,
            tokio::sync::oneshot::Sender<Vec<automerge_protocol::UncompressedChange>>,
        )>,
        mut new_changes: tokio::sync::broadcast::Receiver<
            Vec<automerge_protocol::UncompressedChange>,
        >,
        apply_changes: tokio::sync::mpsc::Sender<Vec<automerge_protocol::UncompressedChange>>,
    ) {
        let peer_hashes = Arc::new(Mutex::new(HashSet::new()));

        let send_msg_1 = send_msg.clone();
        let send_msg_2 = send_msg.clone();
        let send_msg_3 = send_msg.clone();

        let peer_hashes_1 = peer_hashes.clone();
        let peer_hashes_2 = peer_hashes.clone();

        // task for receiving messages
        let recv = tokio::spawn(async move {
            while let Some(msg) = recv_msg.recv().await {
                match msg {
                    Message::Heads(heads) => {
                        for head in &heads {
                            peer_hashes_1.lock().unwrap().insert(*head);
                        }
                        let changes = Self::get_changes(&get_changes, heads).await;
                        send_msg_1.send(Message::Changes(changes)).await.unwrap();
                    }
                    Message::Changes(changes) => {
                        for change in &changes {
                            if let Some(hash) = change.hash {
                                peer_hashes_1.lock().unwrap().insert(hash);
                            }
                        }
                        apply_changes.send(changes).await.unwrap();
                    }
                }
            }
        });

        // task for new local changes
        let send = tokio::spawn(async move {
            while let Ok(changes) = new_changes.recv().await {
                let changes = changes
                    .into_iter()
                    .filter(|c| {
                        if let Some(hash) = c.hash {
                            !peer_hashes_2.lock().unwrap().contains(&hash)
                        } else {
                            true
                        }
                    })
                    .collect::<Vec<_>>();

                for change in &changes {
                    if let Some(hash) = change.hash {
                        peer_hashes_2.lock().unwrap().insert(hash);
                    }
                }

                send_msg_2.send(Message::Changes(changes)).await.unwrap();
            }
        });

        // task for periodic heads sync
        let interval = tokio::spawn(async move {
            let interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            tokio::pin!(interval);

            loop {
                let heads = Self::get_heads(&get_heads).await;
                send_msg_3.send(Message::Heads(heads)).await.unwrap();
                interval.as_mut().tick().await;
            }
        });

        let (recv, send, interval) = tokio::join![recv, send, interval];
        recv.unwrap();
        send.unwrap();
        interval.unwrap();
    }

    async fn get_heads(
        get_heads: &tokio::sync::mpsc::Sender<
            tokio::sync::oneshot::Sender<Vec<automerge_protocol::ChangeHash>>,
        >,
    ) -> Heads {
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_heads.send(tx).await.unwrap();

        rx.await.unwrap()
    }

    async fn get_changes(
        get_changes: &tokio::sync::mpsc::Sender<(
            Vec<automerge_protocol::ChangeHash>,
            tokio::sync::oneshot::Sender<Vec<automerge_protocol::UncompressedChange>>,
        )>,
        heads: Heads,
    ) -> Changes {
        let (tx, rx) = tokio::sync::oneshot::channel();
        get_changes.send((heads, tx)).await.unwrap();

        rx.await.unwrap()
    }
}

pub type Heads = Vec<automerge_protocol::ChangeHash>;

pub type Changes = Vec<automerge_protocol::UncompressedChange>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Message {
    Heads(Heads),
    Changes(Changes),
}

impl TryFrom<Message> for String {
    type Error = serde_json::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        serde_json::to_string(&value)
    }
}

impl TryFrom<&str> for Message {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for Message {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
