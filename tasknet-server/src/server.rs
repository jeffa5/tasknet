use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{future::join_all, stream::SplitSink, SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::{
    addr::remote,
    hyper::Uri,
    path::FullPath,
    ws::{Message, WebSocket},
    Filter,
};

use crate::options::Options;

type MemoryBackend = Arc<
    Mutex<
        automerge_persistent::PersistentBackend<
            automerge_persistent::MemoryPersister,
            automerge::Backend,
        >,
    >,
>;

fn with_backend(
    backend: MemoryBackend,
) -> impl warp::Filter<Extract = (MemoryBackend,), Error = Infallible> + Clone {
    warp::any().map(move || backend.clone())
}

fn with_watch_channel(
    sender: broadcast::Sender<()>,
) -> impl warp::Filter<
    Extract = ((broadcast::Sender<()>, broadcast::Receiver<()>),),
    Error = Infallible,
> + Clone {
    warp::any().map(move || (sender.clone(), sender.subscribe()))
}

pub async fn run(options: Options) {
    tracing_subscriber::fmt::init();

    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");

    let persistent_backend =
        automerge_persistent::PersistentBackend::<_, automerge::Backend>::load(
            automerge_persistent::MemoryPersister::default(),
        )
        .unwrap();

    let backend = Arc::new(Mutex::new(persistent_backend));

    let (sender, _) = broadcast::channel(1);

    let sync = warp::path("sync")
            .and(warp::ws())
            .and(with_backend(backend))
            .and(with_watch_channel(sender))
            .and(remote())
            .map({
                move |ws: warp::ws::Ws,
                      mut backend: MemoryBackend,
                      (sender, mut receiver): (broadcast::Sender<()>, broadcast::Receiver<()>),
                      address: Option<SocketAddr>| {
                    ws.on_upgrade(move |websocket| async move {
                        let (mut tx, mut rx) = websocket.split();

                        let address = address.unwrap();
                        tracing::debug!("connection from {:?}", address);
                        let peer_id = format!("{:?}", address).into_bytes();

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
                                                    .unwrap()
                                                    .receive_sync_message(peer_id.clone(), sync_message)
                                                    .unwrap();

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

                        backend.lock().unwrap().reset_sync_state(&peer_id)
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
    backend: &mut MemoryBackend,
    peer_id: Vec<u8>,
    address: SocketAddr,
    tx: &mut SplitSink<WebSocket, Message>,
) {
    let message = backend
        .lock()
        .unwrap()
        .generate_sync_message(peer_id.clone())
        .unwrap();
    if let Some(message) = message {
        tracing::debug!("Sending sync message to {:?}", address);
        let binary = Vec::<u8>::from(tasknet_sync::Message::SyncMessage(message));
        let _ = tx.send(warp::ws::Message::binary(binary)).await;
    }
}