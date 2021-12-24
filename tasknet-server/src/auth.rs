use kratos_api::models::Session;
use reqwest::{header::COOKIE, Client};
use tracing::{info, warn};
use warp::{Filter, Rejection};

use crate::server::ApiError;

pub fn auth(kratos_url: String) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::cookie::cookie("ory_kratos_session")
        .and(warp::any().map(move || kratos_url.clone()))
        .and_then(|session_token: String, kratos_url: String| async move {
            // TODO: use ID for lookup in the database to load the document.
            info!(?session_token, "Found session token in cookie");

            let response = Client::new()
                .get(format!("{}/sessions/whoami", kratos_url))
                .header(COOKIE, format!("ory_kratos_session={}", session_token))
                .send()
                .await
                .unwrap();
            if response.status() != reqwest::StatusCode::OK {
                let text = response.text().await.unwrap();
                warn!(response=%text, "Failed to get whoami");
                Err(warp::reject::custom(ApiError::Unauthorized))
            } else {
                let session = response.json::<Session>().await.unwrap();
                assert!(session.active.unwrap_or_default());
                if !session.active.unwrap_or_default() {
                    return Err(warp::reject::custom(ApiError::Unauthorized));
                }
                let id = session.identity.id;
                info!(%id, "Found session id from whoami");
                Ok(id)
            }
        })
}
