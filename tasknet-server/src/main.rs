use std::{
    convert::TryFrom,
    sync::{Arc, Mutex},
};

use automerge::Change;
use futures_util::{SinkExt, StreamExt};
use tracing::warn;
use warp::{filters::ws::Message, Filter};

#[tokio::main]
async fn main() {
    env_logger::init();
    let tasknet_log = warp::log("tasknet::web");
    let sync_log = warp::log("tasknet::sync");
    // use spawn local here with channels to interact with the document?
    let local = tokio::task::LocalSet::new();
    let (get_heads_tx, mut get_heads_rx): (
        tokio::sync::mpsc::Sender<
            tokio::sync::oneshot::Sender<Vec<automerge_protocol::ChangeHash>>,
        >,
        _,
    ) = tokio::sync::mpsc::channel(1);
    let (apply_changes_tx, mut apply_changes_rx): (
        tokio::sync::mpsc::Sender<Vec<automerge_protocol::UncompressedChange>>,
        tokio::sync::mpsc::Receiver<Vec<automerge_protocol::UncompressedChange>>,
    ) = tokio::sync::mpsc::channel(1);
    let (get_changes_tx, mut get_changes_rx): (
        tokio::sync::mpsc::Sender<(
            Vec<automerge_protocol::ChangeHash>,
            tokio::sync::oneshot::Sender<_>,
        )>,
        tokio::sync::mpsc::Receiver<_>,
    ) = tokio::sync::mpsc::channel(1);
    let doc_task = local.run_until(async {
        let doc = Arc::new(Mutex::new(automerge::Backend::init()));
        let doc_clone = doc.clone();
        let heads_task = tokio::task::spawn_local(async move {
            while let Some(heads_tx) = get_heads_rx.recv().await {
                let heads = doc_clone.lock().unwrap().get_heads();
                let _ = heads_tx.send(heads);
            }
        });
        let doc_clone = doc.clone();
        let changes_task = tokio::task::spawn_local(async move {
            while let Some(changes) = apply_changes_rx.recv().await {
                let changes = changes.iter().map(Change::from).collect::<Vec<_>>();
                doc_clone.lock().unwrap().apply_changes(changes).unwrap();
            }
        });
        let doc_clone = doc.clone();
        let get_changes_task = tokio::task::spawn_local(async move {
            while let Some((heads, sender)) = get_changes_rx.recv().await {
                let doc = doc_clone.lock().unwrap();
                let changes: Vec<_> = doc.get_changes(&heads);
                let changes: Vec<_> = changes.iter().map(|c| c.decode()).collect();
                let _ = sender.send(changes);
            }
        });
        let (heads_task, changes_task, get_changes_task) =
            tokio::join![heads_task, changes_task, get_changes_task];
        if let Err(e) = heads_task {
            warn!(error = ?e, "error joining heads_task")
        }
        if let Err(e) = changes_task {
            warn!(error = ?e, "error joining changes task")
        }
        if let Err(e) = get_changes_task {
            warn!(error = ?e, "error joining get_changes task")
        }
    });
    let (new_changes_tx, _new_changes_rx) = tokio::sync::broadcast::channel(1);
    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .map({
                move |ws: warp::ws::Ws| {
                    let get_heads_tx = get_heads_tx.clone();
                    let apply_changes_tx = apply_changes_tx.clone();
                    let get_changes_tx = get_changes_tx.clone();
                    let new_changes_tx = new_changes_tx.clone();
                    let new_changes_rx = new_changes_tx.subscribe();
                    ws.on_upgrade(|websocket| async move {
                        let (mut tx, mut rx) = websocket.split();

                        let (msgs_out_tx, mut msgs_out_rx) = tokio::sync::mpsc::channel(1);
                        let (msgs_in_tx, msgs_in_rx) = tokio::sync::mpsc::channel(1);

                        tokio::spawn(async move {
                            while let Some(msg) = msgs_out_rx.recv().await {
                                let text = String::try_from(msg).unwrap();
                                let _ = tx.send(Message::text(text)).await;
                            }
                        });

                        tokio::spawn(async move {
                            while let Some(Ok(msg)) = rx.next().await {
                                let text_msg = msg.to_str();
                                if let Ok(msg) = text_msg {
                                    if let Ok(msg) = tasknet_sync::Message::try_from(msg) {
                                        let _ = msgs_in_tx.send(msg).await;
                                    } else {
                                        eprintln!("unexpected message {:?}", msg)
                                    }
                                } else {
                                    eprintln!("found non text msg: {:?}", msg)
                                }
                            }
                        });

                        tasknet_sync::Connection::handle(
                            msgs_out_tx,
                            msgs_in_rx,
                            get_heads_tx,
                            get_changes_tx,
                            new_changes_tx,
                            new_changes_rx,
                            apply_changes_tx,
                        )
                        .await;
                    })
                }
            })
            .with(sync_log));
    tokio::join![doc_task, warp::serve(routes).run(([127, 0, 0, 1], 8080))];
}
