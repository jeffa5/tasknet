use async_session::MemoryStore;
use google::Google;
use std::collections::HashMap;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::debug;
use tracing::info;

use axum::{routing::get, Router};
use clap::Parser;
use tokio::sync::Mutex;

mod auth;
mod config;
mod google;
mod server;

#[derive(Debug, clap::Parser)]
struct ServerOptions {
    #[clap(long, short, default_value = "config.json")]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() {
    let options = ServerOptions::parse();

    tracing_subscriber::fmt::init();

    debug!(?options, "Parsed CLI options");

    let config = config::ServerConfig::load(&options.config_file);

    debug!(?config, "Loaded config file");

    let (changed, _) = tokio::sync::broadcast::channel(1);

    let port = config.port;

    let google = if let Some(config) = config.google.as_ref() {
        Some(Google::new(config).await)
    } else {
        None
    };

    let app = Router::new()
        .route("/sync", get(server::sync_handler))
        .route("/auth/providers", get(auth::providers))
        .route("/auth/google/sign_in", get(google::sign_in_handler))
        .route("/auth/google/sign_out", get(google::sign_out_handler))
        .route("/auth/google/callback", get(google::callback_handler))
        .nest_service(
            "/",
            ServeDir::new(&config.serve_dir).not_found_service(ServeFile::new("index.html")),
        )
        .with_state(Arc::new(Mutex::new(server::Server {
            documents: HashMap::new(),
            changed,
            config,
            google,
            sessions: MemoryStore::new(),
        })))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Listening on http://localhost:{}", port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, starting graceful shutdown");
}
