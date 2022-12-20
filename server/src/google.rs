use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::Server;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GoogleConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scopes: Vec<String>,
    base_uri: String,
}

pub async fn handler(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
    if let Some(config) = server.lock().await.config.google.as_ref() {
        Redirect::to(&config.base_uri)
    } else {
        Redirect::to("/")
    }
}
