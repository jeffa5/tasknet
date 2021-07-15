use automerge_backend::SyncState;
use futures_util::{SinkExt, StreamExt};
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();
    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");

    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .map({
                move |ws: warp::ws::Ws| {
                    ws.on_upgrade(|websocket| async move {
                        let (mut tx, mut rx) = websocket.split();

                        let mut sync_state = SyncState::default();
                        let mut backend = automerge::Backend::new();

                        while let Some(Ok(msg)) = rx.next().await {
                            if msg.is_binary() {
                                let bytes = msg.into_bytes();
                                let message = tasknet_sync::Message::from(bytes);
                                match message {
                                    tasknet_sync::Message::SyncMessage(sync_message) => {
                                        let _patch = backend
                                            .receive_sync_message(&mut sync_state, sync_message)
                                            .unwrap();
                                        let message =
                                            backend.generate_sync_message(&mut sync_state);
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
