use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use warp::{addr::remote, Filter};

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

#[tokio::main]
async fn main() {
    env_logger::init();
    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");

    let persistent_backend =
        automerge_persistent::PersistentBackend::<_, automerge::Backend>::load(
            automerge_persistent::MemoryPersister::default(),
        )
        .unwrap();

    let backend = Arc::new(Mutex::new(persistent_backend));

    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .and(with_backend(backend))
            .and(remote())
            .map({
                move |ws: warp::ws::Ws, backend: MemoryBackend, address: Option<SocketAddr>| {
                    ws.on_upgrade(move |websocket| async move {
                        let (mut tx, mut rx) = websocket.split();

                        let address = address.unwrap();
                        println!("connection from {:?}", address);
                        let peer_id = format!("{:?}", address).into_bytes();

                        // Send a message to the client first. If they don't have any changes
                        // then their generate_sync_message will be None so they won't have
                        // anything to send.
                        let message = backend
                            .lock()
                            .unwrap()
                            .generate_sync_message(peer_id.clone())
                            .unwrap();
                        if let Some(message) = message {
                            println!("Sending initial sync message to {:?}", address);
                            let binary =
                                Vec::<u8>::from(tasknet_sync::Message::SyncMessage(message));
                            tx.send(warp::ws::Message::binary(binary)).await.unwrap();
                        }

                        while let Some(Ok(msg)) = rx.next().await {
                            if msg.is_binary() {
                                let bytes = msg.into_bytes();
                                let message = tasknet_sync::Message::from(bytes);
                                match message {
                                    tasknet_sync::Message::SyncMessage(sync_message) => {
                                        println!("Received sync message from {:?}", address);
                                        let _patch = backend
                                            .lock()
                                            .unwrap()
                                            .receive_sync_message(peer_id.clone(), sync_message)
                                            .unwrap();
                                        let message = backend
                                            .lock()
                                            .unwrap()
                                            .generate_sync_message(peer_id.clone())
                                            .unwrap();
                                        if let Some(message) = message {
                                            let binary = Vec::<u8>::from(
                                                tasknet_sync::Message::SyncMessage(message),
                                            );
                                            tx.send(warp::ws::Message::binary(binary))
                                                .await
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    })
                }
            })
            .with(sync_log));

    println!("Serving page on http://localhost:8080/tasknet");

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
