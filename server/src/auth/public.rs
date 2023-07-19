use std::sync::Arc;

use async_session::{Session, SessionStore};
use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use tasknet_shared::cookies::{AUTH_PROVIDER_COOKIE, DOCUMENT_ID_COOKIE, SESSION_COOKIE};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{auth::UserSessionData, server::Server};

use super::{clear_session_cookies, UserIdFromSession};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PublicConfig {
    client_id: String,
    client_secret: String,
    issuer_uri: String,
    redirect_uri: String,
    scopes: Vec<String>,
}

pub async fn sign_in_handler(
    Query(query): Query<DocId>,
    State(server): State<Arc<Mutex<Server>>>,
) -> impl IntoResponse {
    debug!("Public sign in handler");
    let server = server.lock().await;

    let mut headers = HeaderMap::new();

    let doc_id = if query.doc_id.is_empty() {
        // generate one
        uuid::Uuid::new_v4()
    } else {
        let Ok(doc_id) = query.doc_id.parse::<uuid::Uuid>() else {
            // Expected a uuid
            // TODO: indicate error to user
            return Err((StatusCode::BAD_REQUEST, Redirect::to("/")));
        };
        doc_id
    };

    // Create a new session filled with user data
    let mut session = Session::new();
    let user_data = UserSessionData::Public {
        doc_id: doc_id.to_string(),
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
        format!("{}={}; Path=/", AUTH_PROVIDER_COOKIE, "public"),
        format!("{}={}; Path=/", DOCUMENT_ID_COOKIE, user_data.doc_id()),
    ];

    // Set cookies
    for cookie in cookies {
        headers.append(SET_COOKIE, cookie.parse().unwrap());
    }

    Ok((headers, Redirect::to("/")))
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
pub struct DocId {
    doc_id: String,
}
