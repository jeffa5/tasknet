use automerge_backend::SyncMessage;
use automerge_protocol::Patch;

#[derive(Default, Debug)]
pub struct Backend {
    inner: automerge::Backend,
}

impl Backend {
    pub async fn load(postgres_client: tokio_postgres::Client, doc_id: Vec<u8>) -> Self {
        // get changes
        let change_rows = postgres_client
            .query("SELECT data FROM changes WHERE doc_id = $1", &[&doc_id])
            .await
            .unwrap();

        change_rows.into_iter().map(|row| row.get("doc_id"));

        // get document

        // build document
        todo!()
    }

    pub async fn receive_sync_message(
        &mut self,
        peer_id: Vec<u8>,
        sync_message: SyncMessage,
    ) -> Option<Patch> {
        todo!()
    }

    pub async fn generate_sync_message(&mut self, peer_id: Vec<u8>) -> Option<SyncMessage> {
        todo!()
    }

    pub async fn reset_sync_state(&mut self, peer_id: &[u8]) {
        todo!()
    }
}
