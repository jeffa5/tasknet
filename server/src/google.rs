use axum::extract::TypedHeader;
use axum::headers::Cookie;
use axum::{http::request::Parts, RequestPartsExt};
use std::sync::Arc;

use async_session::{async_trait, Session, SessionStore};
use axum::{
    extract::{FromRequestParts, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use openid::{DiscoveredClient, Options, Token};
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, info};

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

pub async fn sign_in_handler(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
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

pub struct UserIdFromSession {
    pub session_cookie: String,
    pub session_data: UserSessionData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSessionData {
    google_id: String,
}

#[async_trait]
impl FromRequestParts<Arc<Mutex<Server>>> for UserIdFromSession {
    type Rejection = (HeaderMap, Redirect);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<Mutex<Server>>,
    ) -> Result<Self, Self::Rejection> {
        let server = state.lock().await;

        let cookie: Option<TypedHeader<Cookie>> = parts.extract().await.unwrap();

        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(SESSION_COOKIE))
            .unwrap_or_default();

        let mut headers = HeaderMap::new();
        // return the new created session cookie for client
        if session_cookie.is_empty() {
            debug!("found an empty session cookie");
            return Err((headers, Redirect::to("/")));
        }

        // continue to decode the session cookie
        let user_data = if let Some(session) = server
            .sessions
            .load_session(session_cookie.to_owned())
            .await
            .unwrap()
        {
            if let Some(user_data) = session.get::<UserSessionData>("user_data") {
                debug!(
                    "UserIdFromSession: session decoded success, user_data={:?}",
                    user_data
                );
                user_data
            } else {
                debug!("Failed to get user_data from session");
                return Err((headers, Redirect::to("/")));
            }
        } else {
            debug!(
                "UserIdFromSession: err session not exists in store, {}={}",
                SESSION_COOKIE, session_cookie
            );

            clear_session_cookies(&mut headers).await;

            return Err((headers, Redirect::to("/")));
        };

        Ok(UserIdFromSession {
            session_cookie: session_cookie.to_owned(),
            session_data: user_data,
        })
    }
}

pub async fn clear_session_cookies(headers: &mut HeaderMap) {
    let cookies = vec![
        format!("{}=; Path=/", SESSION_COOKIE),
        format!("{}=; Path=/", AUTH_PROVIDER_COOKIE),
    ];

    // Set cookies
    for cookie in cookies {
        debug!(cookie, "Setting cookie");
        headers.append(SET_COOKIE, cookie.parse().unwrap());
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
    clear_session_cookies(&mut headers).await;

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
            let user_data = UserSessionData {
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
