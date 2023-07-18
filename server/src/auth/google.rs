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
use tracing::debug;

use crate::server::Server;

use super::UserIdFromSession;

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

pub async fn sign_in_handler(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
    let server = server.lock().await;
    if let (Some(state), Some(config)) = (server.google.as_ref(), server.config.google.as_ref()) {
        debug!("google auth sign in");

        let auth_url: String = state
            .client
            .auth_url(&Options {
                scope: Some(config.scopes.join(" ")),
                ..Default::default()
            })
            .into();

        debug!("Redirecting to {}", auth_url);
        Redirect::to(auth_url.as_ref())
    } else {
        Redirect::to("/")
    }
}

pub async fn sign_out_handler(
    user: UserIdFromSession,
    State(server): State<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    let server = server.lock().await;

    // remove session
    if let Ok(Some(session)) = server.sessions.load_session(user.session_cookie).await {
        server.sessions.destroy_session(session).await.unwrap();
    }

    let mut headers = HeaderMap::new();
    // clear cookies
    super::clear_session_cookies(&mut headers).await;

    (headers, Redirect::to("/"))
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

pub async fn callback_handler(
    Query(query): Query<AuthRequest>,
    State(server): State<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    debug!("Google auth callback");

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
            debug!("token: {:?}", id_token);
            let payload = id_token.payload().unwrap();

            // Create a new session filled with user data
            let mut session = Session::new();
            let user_data = super::UserSessionData::Google {
                google_id: payload.sub.clone(),
            };
            session.insert("user_data", &user_data).unwrap();

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
                    super::SESSION_COOKIE,
                    session_cookie
                ),
                format!("{}={}; Path=/", super::AUTH_PROVIDER_COOKIE, "google"),
            ];

            // Set cookies
            for cookie in cookies {
                headers.append(SET_COOKIE, cookie.parse().unwrap());
            }
        }
    }

    (headers, Redirect::to("/"))
}
