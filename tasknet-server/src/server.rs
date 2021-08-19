use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};

use futures_util::{future::join_all, stream::SplitSink, SinkExt, StreamExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio_postgres::NoTls;
use warp::{
    addr::remote,
    hyper::Uri,
    path::FullPath,
    ws::{Message, WebSocket},
    Filter,
};

use crate::{backend::Backend, options::Options};

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

async fn connect_to_db(options: &Options) -> tokio_postgres::Client {
    let mut postgres_config = tokio_postgres::Config::default();
    postgres_config
        .port(options.db_port)
        .host(&options.db_host)
        .dbname(&options.db_name)
        .user(&options.db_user)
        .password(&options.db_password);

    let mut backoff = 0;
    let retry_interval = 100;
    let (postgres_client, connection) = loop {
        match postgres_config.connect(NoTls).await {
            Ok(v) => break v,
            Err(e) => {
                tracing::warn!(error=e, "Failed to connect to database");
                backoff += 1;
                backoff = std::cmp::min(backoff, 4);
                let duration_millis =
                    retry_interval * rand::thread_rng().gen_range(0..(2_u64.pow(backoff)));
                tokio::time::sleep(Duration::from_millis(duration_millis)).await
            }
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error=?e, "connection closed");
        }
    });

    postgres_client
}

#[derive(Debug, Serialize, Deserialize)]
struct SyncQueryOptions {
    doc_id: uuid::Uuid,
}

pub async fn run(options: Options) {
    tracing_subscriber::fmt::init();

    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");

    let postgres_client = Arc::new(connect_to_db(&options).await);

    let backends = Arc::new(Mutex::new(HashMap::new()));

    let (sender, _) = broadcast::channel(1);

    let sync = warp::path("sync")
            .and(warp::query())
            .and(warp::ws())
            .and(with_backends(backends))
            .and(with_db_client(postgres_client))
            .and(with_watch_channel(sender))
            .and(remote())
            .map({
                move | query_params:SyncQueryOptions,
                    ws: warp::ws::Ws,
                      backends: DBBackends,
                      db_client : Arc<tokio_postgres::Client>,
                      (sender, mut receiver): (broadcast::Sender<()>, broadcast::Receiver<()>),
                      address: Option<SocketAddr>| {
                    ws.on_upgrade(move |websocket| async move {
                        let (mut tx, mut rx) = websocket.split();

                        let address = address.unwrap();
                        tracing::info!(?address, doc_id=?query_params.doc_id, "New sync connection");
                        let peer_id = format!("{:?}", address).into_bytes();

                        let mut backend = backends.lock().await.entry(query_params.doc_id.as_bytes().to_vec()).or_insert(Arc::new(Mutex::new(Backend::load(db_client, query_params.doc_id.as_bytes().to_vec()).await))).clone();

                        // Send a message to the client first. If they don't have any changes
                        // then their generate_sync_message will be None so they won't have
                        // anything to send.
                        send_message(&mut backend,  peer_id.clone(), address, &mut tx).await;

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
            })
            .with(sync_log);

    let static_files_dir = options.static_files_dir.join("tasknet");
    let statics = warp::any().and(warp::fs::dir(static_files_dir).with(tasknet_log));

    let root_redirect = warp::path::end().map(|| {
        warp::redirect({
            tracing::info!("redirecting root to path /tasknet");
            Uri::from_static("/tasknet")
        })
    });

    let routes = warp::get()
        .and(warp::path("tasknet"))
        .and(sync.or(statics))
        .or(root_redirect);

    let https_listen_address = options.https_listen_address;

    let http_listen_address = options.http_listen_address;
    let https_listen_address_2 = options.https_listen_address;

    let tls_server = tokio::spawn(async move {
        warp::serve(routes)
            .tls()
            .cert_path(options.cert_file)
            .key_path(options.key_file)
            .run(https_listen_address)
            .await;
    });

    // redirect to https
    let http_server = tokio::spawn(async move {
        warp::serve(warp::path::full().map(move |path: FullPath| {
            warp::redirect({
                let address = format!("https://{}{}", https_listen_address_2, path.as_str());
                tracing::warn!("redirecting to {:?}", address);
                // path always starts with '/', even if it was empty
                Uri::from_maybe_shared(address).unwrap()
            })
        }))
        .run(http_listen_address)
        .await;
    });

    join_all(vec![tls_server, http_server]).await;
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
