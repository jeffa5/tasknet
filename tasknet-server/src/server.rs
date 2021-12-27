use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use reqwest::redirect::Policy;
use tokio::{
    select,
    signal::unix::SignalKind,
    sync::{broadcast, watch, Mutex},
};
use tracing::{info, warn};
use warp::{
    addr::remote,
    hyper::Uri,
    reject::MissingCookie,
    reply,
    ws::{Message, WebSocket},
    Filter, Rejection, Reply,
};
use warp_reverse_proxy::{reverse_proxy_filter, CLIENT as PROXY_CLIENT};

use crate::{auth::auth, backend::Backend, db::connect_to_db, options::Options};

#[derive(Debug)]
pub(crate) enum ApiError {
    Unauthorized,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.is_not_found() {
        Ok(reply::with_status(
            "NOT_FOUND".to_owned(),
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = err.find::<ApiError>() {
        match e {
            ApiError::Unauthorized => Ok(reply::with_status(
                "UNAUTHORIZED".to_owned(),
                warp::http::StatusCode::UNAUTHORIZED,
            )),
        }
    } else if let Some(e) = err.find::<MissingCookie>() {
        Ok(reply::with_status(
            format!("BAD_REQUEST, missing cookie {}", e.name()),
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else {
        eprintln!("unhandled rejection: {:?}", err);
        Ok(reply::with_status(
            "INTERNAL_SERVER_ERROR".to_owned(),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

impl warp::reject::Reject for ApiError {}

type DBBackends = Arc<Mutex<HashMap<Vec<u8>, Arc<Mutex<Backend>>>>>;

fn with_backends(
    backend: DBBackends,
) -> impl warp::Filter<Extract = (DBBackends,), Error = Infallible> + Clone {
    warp::any().map(move || backend.clone())
}

fn with_db_client(
    client: Arc<tokio_postgres::Client>,
) -> impl warp::Filter<Extract = (Arc<tokio_postgres::Client>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_watch_channel(
    sender: broadcast::Sender<()>,
) -> impl warp::Filter<
    Extract = ((broadcast::Sender<()>, broadcast::Receiver<()>),),
    Error = Infallible,
> + Clone {
    warp::any().map(move || (sender.clone(), sender.subscribe()))
}

fn handle_sync_connection(
    user_id: String,
    ws: warp::ws::Ws,
    backends: DBBackends,
    db_client: Arc<tokio_postgres::Client>,
    (sender, mut receiver): (broadcast::Sender<()>, broadcast::Receiver<()>),
    address: Option<SocketAddr>,
) -> impl Reply {
    ws.on_upgrade(move |websocket| async move {
        let (mut tx, mut rx) = websocket.split();

        let address = address.unwrap();
        tracing::info!(?address, ?user_id, "New sync connection");
        let peer_id = format!("{:?}", address).into_bytes();

        let mut backend = backends
            .lock()
            .await
            .entry(user_id.as_bytes().to_vec())
            .or_insert(Arc::new(Mutex::new(
                Backend::load(db_client, user_id.as_bytes().to_vec()).await,
            )))
            .clone();

        // Send a message to the client first. If they don't have any changes
        // then their generate_sync_message will be None so they won't have
        // anything to send.
        send_message(&mut backend, peer_id.clone(), address, &mut tx).await;

        loop {
            tokio::select! {
                Some(Ok(msg)) = rx.next() => {
                    if msg.is_binary() {
                        let bytes = msg.into_bytes();
                        let message = tasknet_sync::Message::from(bytes);
                        match message {
                            tasknet_sync::Message::SyncMessage(sync_message) => {
                                tracing::debug!("Received sync message from {:?}", address);
                                let patch = backend
                                    .lock()
                                    .await
                                    .receive_sync_message(peer_id.clone(), sync_message).await;

                                if patch.is_some() {
                                    sender.send(()).unwrap();
                                }

                                send_message(&mut backend, peer_id.clone(), address, &mut tx).await
                            }
                        }
                    } else if msg.is_close() {
                        tracing::debug!("close");
                        break
                    } else if msg.is_ping() {
                        // nothing to do as handled by underlying implementation
                    } else {
                        tracing::warn!("unhandled message {:?}", msg);
                    }
                },

                Ok(()) = receiver.recv() => {
                    send_message(&mut backend, peer_id.clone(), address, &mut tx).await
                },

                else => break,
            }
        }

        backend.lock().await.reset_sync_state(&peer_id).await;
    })
}

pub async fn run(options: Options) {
    tracing_subscriber::fmt::init();

    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");
    let proxy_log = warp::log("tasknet::proxy");

    let postgres_client = Arc::new(connect_to_db(&options).await);

    let backends = Arc::new(Mutex::new(HashMap::new()));

    let (sender, _) = broadcast::channel(1);

    let sync = warp::path("sync")
        .and(auth(options.kratos_url.clone()))
        .and(warp::ws())
        .and(with_backends(backends))
        .and(with_db_client(postgres_client))
        .and(with_watch_channel(sender))
        .and(remote())
        .map(handle_sync_connection)
        .with(sync_log);

    let static_files_dir = options.static_files_dir.join("tasknet");
    let statics = warp::any().and(warp::fs::dir(static_files_dir).with(tasknet_log));

    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("client couldn't be built");
    PROXY_CLIENT.set(client).expect("client couldn't be set");

    let kratos_proxy = warp::path::path("kratos")
        .and(reverse_proxy_filter(
            "kratos".to_string(),
            options.kratos_url,
        ))
        .with(proxy_log);

    let root_redirect = warp::path::end().map(|| {
        warp::redirect({
            tracing::info!("redirecting root to path /tasknet");
            Uri::from_static("/tasknet")
        })
    });

    let routes = warp::get()
        .and(warp::path("tasknet"))
        .and(sync.or(statics))
        .or(kratos_proxy)
        .or(root_redirect)
        .recover(handle_rejection);

    let http_listen_address = options.http_listen_address;

    let (shutdown_tx, shutdown_rx) = watch::channel(());

    let mut shutdown_rx_1 = shutdown_rx.clone();
    let (_, http_server) =
        warp::serve(routes).bind_with_graceful_shutdown(http_listen_address, async move {
            let _ = shutdown_rx_1.changed().await;
            info!("Shutting down http server");
        });
    let http_server = tokio::spawn(async move {
        http_server.await;
    });
    info!(address = %http_listen_address, "Started http server");

    let mut sig_term = tokio::signal::unix::signal(SignalKind::terminate()).unwrap();
    select! {
        s = sig_term.recv() => match s {
            Some(()) => {}
            None => {
                warn!("Failed to listen for shutdown terminate signal");
            }
        },
        c = tokio::signal::ctrl_c() => match c {
            Ok(()) => {}
            Err(err) => {
                warn!(%err,"Failed to listen for shutdown interrupt signal");
            }
        }
    };
    info!("Shutting down");
    let _ = shutdown_tx.send(());

    http_server.await.unwrap()
}

async fn send_message(
    backend: &mut Arc<Mutex<Backend>>,
    peer_id: Vec<u8>,
    address: SocketAddr,
    tx: &mut SplitSink<WebSocket, Message>,
) {
    let message = backend
        .lock()
        .await
        .generate_sync_message(peer_id.clone())
        .await;
    if let Some(message) = message {
        tracing::debug!("Sending sync message to {:?}", address);
        let binary = Vec::<u8>::from(tasknet_sync::Message::SyncMessage(message));
        let _ = tx.send(warp::ws::Message::binary(binary)).await;
    }
}
