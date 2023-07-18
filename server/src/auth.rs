use reqwest::header::SET_COOKIE;
use std::sync::Arc;

use async_session::{async_trait, SessionStore};
use axum::extract::TypedHeader;
use axum::headers::Cookie;
use axum::response::{Redirect, Response};
use axum::{
    extract::{FromRequestParts, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use axum::{http::request::Parts, http::StatusCode, RequestPartsExt};
use serde::{Deserialize, Serialize};
use tasknet_shared::providers::{ProviderDefault, Providers};
use tokio::sync::Mutex;
use tracing::debug;

use crate::server::Server;

pub mod google;
pub mod public;

pub const SESSION_COOKIE: &str = "session";
pub const AUTH_PROVIDER_COOKIE: &str = "auth-provider";

pub async fn providers(State(server): State<Arc<Mutex<Server>>>) -> impl IntoResponse {
    let mut providers = Providers::default();
    // always include public
    providers.public = ProviderDefault { enabled: true };
    let server = server.lock().await;
    providers.google = ProviderDefault {
        enabled: server.google.is_some(),
    };
    Json(providers)
}

pub struct UserIdFromSession {
    pub session_cookie: String,
    pub session_data: UserSessionData,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserSessionData {
    Google { google_id: String },
    Public { doc_id: String },
}

impl UserSessionData {
    pub fn doc_id(&self) -> &str {
        match self {
            UserSessionData::Google { google_id } => google_id,
            UserSessionData::Public { doc_id } => doc_id,
        }
    }
}

#[async_trait]
impl FromRequestParts<Arc<Mutex<Server>>> for UserSessionData {
    type Rejection = (HeaderMap, Response);

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

        let error_response = if let Some("websocket") =
            parts.headers.get("upgrade").and_then(|hv| hv.to_str().ok())
        {
            StatusCode::UNAUTHORIZED.into_response()
        } else {
            Redirect::to("/").into_response()
        };

        let mut headers = HeaderMap::new();
        // return the new created session cookie for client
        if session_cookie.is_empty() {
            debug!("found an empty session cookie");
            return Err((headers, error_response));
        }

        // continue to decode the session cookie
        if let Some(session) = server
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
                return Ok(user_data);
            } else {
                debug!("Failed to get user_data from session");
                return Err((headers, error_response));
            }
        } else {
            debug!(
                "UserIdFromSession: err session not exists in store, {}={}",
                SESSION_COOKIE, session_cookie
            );

            clear_session_cookies(&mut headers).await;

            return Err((headers, error_response));
        }
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
