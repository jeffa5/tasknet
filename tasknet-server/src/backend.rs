use std::{collections::HashMap, sync::Arc};

use automerge::Change;
use automerge_backend::{SyncMessage, SyncState};
use automerge_protocol::Patch;

#[derive(Debug)]
pub struct Backend {
    doc_id: Vec<u8>,
    inner: automerge::Backend,
    sync_states: HashMap<Vec<u8>, SyncState>,
    db_client: Arc<tokio_postgres::Client>,
}

impl Backend {
    pub async fn load(postgres_client: Arc<tokio_postgres::Client>, doc_id: Vec<u8>) -> Self {
        // get changes
        let change_rows = postgres_client
            .query("SELECT data FROM changes WHERE doc_id = $1", &[&doc_id])
            .await
            .unwrap();

        let changes: Result<Vec<_>, _> = change_rows
            .into_iter()
            .map(|row| row.get("data"))
            .map(Change::from_bytes)
            .collect();

        let changes = changes.unwrap();

        let mut backend = automerge::Backend::new();
        backend.load_changes(changes).unwrap();

        Self {
            doc_id,
            inner: backend,
            sync_states: HashMap::new(),
            db_client: postgres_client,
        }
    }

    pub async fn receive_sync_message(
        &mut self,
        peer_id: Vec<u8>,
        sync_message: SyncMessage,
    ) -> Option<Patch> {
        // get the sync state from hashmap, or load from db
        let mut sync_state = self.sync_states.entry(peer_id).or_default();

        let heads = self.inner.get_heads();

        let patch = self
            .inner
            .receive_sync_message(&mut sync_state, sync_message)
            .unwrap();

        let new_changes = self.inner.get_changes(&heads);

        for change in new_changes {
            self.db_client
                .execute(
                    "INSERT INTO changes (doc_id, hash, data) VALUES ($1, $2, $3)",
                    &[&self.doc_id, &&change.hash.0[..], &change.raw_bytes()],
                )
                .await
                .unwrap();
        }

        patch
    }

    pub async fn generate_sync_message(&mut self, peer_id: Vec<u8>) -> Option<SyncMessage> {
        // get the sync state from hashmap, or load from db, or default
        let sync_state = self.sync_states.entry(peer_id).or_default();

        self.inner.generate_sync_message(sync_state)
    }

    pub async fn reset_sync_state(&mut self, peer_id: &[u8]) {
        self.sync_states.remove(peer_id);
    }
}
