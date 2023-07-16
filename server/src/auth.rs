use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use sync::providers::Providers;
use tokio::sync::Mutex;

use crate::server::Server;

pub async fn providers(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
    let mut providers = Providers::default();
    let server = server.lock().await;
    if server.google.is_some() {
        providers.google = Some(());
    }
    Json(providers)
}
