use std::sync::{Arc, Mutex};

use automerge::Change;
use futures_util::{FutureExt, SinkExt, StreamExt};
use warp::{filters::ws::Message, Filter};

#[tokio::main]
async fn main() {
    env_logger::init();
    let tasknet_log = warp::log("tasknet::tasknet");
    let sync_log = warp::log("tasknet::sync");
    let doc = Arc::new(Mutex::new(automerge::Backend::init()));
    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|websocket| async {
                    eprintln!("websocket connection");
                    let (mut tx, mut rx) = websocket.split();
                    let heads: Vec<automerge_protocol::ChangeHash> = Vec::new();
                    // heads = doc.clone().lock().unwrap().get_heads();
                    tx.send(Message::text(serde_json::to_string(&heads).unwrap()))
                        .await
                        .unwrap();
                    while let Some(changes) = rx.next().await {
                        if let Ok(msg) = changes {
                            if let Ok(text) = msg.to_str() {
                                let changes: Vec<automerge_protocol::UncompressedChange> =
                                    serde_json::from_str(text).unwrap();
                                let changes: Vec<_> = changes.iter().map(Change::from).collect();
                                eprintln!("changes {:?}", changes);
                                // doc.clone().lock().unwrap().apply_changes(changes).unwrap();
                            }
                        }
                    }
                })
            })
            .with(sync_log));
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
