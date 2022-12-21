use std::sync::Arc;

use async_session::{Session, SessionStore};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use openid::{DiscoveredClient, Options, Token};
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::info;

use crate::Server;

pub const SESSION_COOKIE: &str = "session";
pub const AUTH_PROVIDER_COOKIE: &str = "auth-provider";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GoogleConfig {
    client_id: String,
    client_secret: String,
    issuer_uri: String,
    redirect_uri: String,
    scopes: Vec<String>,
}

pub struct Google {
    client: DiscoveredClient,
}

impl Google {
    pub async fn new(config: &GoogleConfig) -> Self {
        let client = DiscoveredClient::discover(
            config.client_id.clone(),
            config.client_secret.clone(),
            Some(config.redirect_uri.clone()),
            reqwest::Url::parse(&config.issuer_uri).unwrap(),
        )
        .await
        .unwrap();
        Self { client }
    }
}

pub async fn handler(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
    let server = server.lock().await;
    if let (Some(state), Some(config)) = (server.google.as_ref(), server.config.google.as_ref()) {
        info!("google auth sign in");

        let auth_url: String = state
            .client
            .auth_url(&Options {
                scope: Some(config.scopes.join(" ")),
                ..Default::default()
            })
            .into();

        info!("Redirecting to {}", auth_url);
        Redirect::to(auth_url.as_ref())
    } else {
        Redirect::to("/")
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

pub async fn callback_handler(
    Query(query): Query<AuthRequest>,
    State(server): State<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    info!("Google auth callback");

    let server = server.lock().await;

    let mut headers = HeaderMap::new();

    if let Some(state) = server.google.as_ref() {
        let mut token: Token = state
            .client
            .request_token(&query.code)
            .await
            .unwrap()
            .into();

        if let Some(id_token) = token.id_token.as_mut() {
            state.client.decode_token(id_token).unwrap();
            state.client.validate_token(id_token, None, None).unwrap();
            info!("token: {:?}", id_token);
            let payload = id_token.payload().unwrap();

            // Create a new session filled with user data
            let mut session = Session::new();
            session.insert("google_id", &payload.sub).unwrap();

            // Store session and get corresponding cookie
            let session_cookie = server
                .sessions
                .store_session(session)
                .await
                .unwrap()
                .unwrap();

            // Build the cookie
            let cookies = vec![
                format!(
                    "{}={}; SameSite=Lax; Path=/",
                    SESSION_COOKIE, session_cookie
                ),
                format!("{}={}; Path=/", AUTH_PROVIDER_COOKIE, "google"),
            ];

            // Set cookies
            for cookie in cookies {
                headers.append(SET_COOKIE, cookie.parse().unwrap());
            }
        }
    }

    (headers, Redirect::to("/"))
}
