use std::sync::{Arc, Mutex};

use automerge::Change;
use futures_util::{FutureExt, SinkExt, StreamExt};
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
    let (apply_changes_tx, mut apply_changes_rx) = tokio::sync::mpsc::channel(1);
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
                heads_tx.send(heads).unwrap()
            }
        });
        let doc_clone = doc.clone();
        let changes_task = tokio::task::spawn_local(async move {
            while let Some(changes) = apply_changes_rx.recv().await {
                doc_clone.lock().unwrap().apply_changes(changes).unwrap();
            }
        });
        let doc_clone = doc.clone();
        let get_changes_task = tokio::task::spawn_local(async move {
            while let Some((heads, sender)) = get_changes_rx.recv().await {
                let doc = doc_clone.lock().unwrap();
                let changes: Vec<_> = doc.get_changes(&heads);
                let changes: Vec<_> = changes.iter().map(|c| c.decode()).collect();
                sender.send(changes).unwrap();
            }
        });
        let (heads_task, changes_task, get_changes_task) =
            tokio::join![heads_task, changes_task, get_changes_task];
        heads_task.unwrap();
        changes_task.unwrap();
        get_changes_task.unwrap();
    });
    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let get_heads_tx = get_heads_tx.clone();
                let apply_changes_tx = apply_changes_tx.clone();
                let get_changes_tx = get_changes_tx.clone();
                ws.on_upgrade(|websocket| async move {
                    let (mut tx, mut rx) = websocket.split();
                    let (msgs_tx, mut msgs_rx) = tokio::sync::mpsc::channel(1);
                    let msgs_tx_clone = msgs_tx.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = msgs_rx.recv().await {
                            tx.send(Message::text(msg)).await.unwrap()
                        }
                    });
                    tokio::spawn(async move {
                        let interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
                        tokio::pin!(interval);

                        loop {
                            interval.as_mut().tick().await;
                            let (heads_tx, heads_rx) = tokio::sync::oneshot::channel();
                            get_heads_tx.send(heads_tx).await.unwrap();
                            let heads = heads_rx.await.unwrap();
                            eprintln!("sending {} heads", heads.len());
                            msgs_tx
                                .send(serde_json::to_string(&heads).unwrap())
                                .await
                                .unwrap();
                        }
                    });

                    while let Some(changes) = rx.next().await {
                        if let Ok(msg) = changes {
                            if let Ok(text) = msg.to_str() {
                                if let Ok(heads) = serde_json::from_str::<
                                    Vec<automerge_protocol::ChangeHash>,
                                >(text)
                                {
                                    eprintln!("received heads");
                                    let (changes_tx, changes_rx) = tokio::sync::oneshot::channel();
                                    get_changes_tx.send((heads, changes_tx)).await.unwrap();
                                    let changes: Vec<automerge_protocol::UncompressedChange> =
                                        changes_rx.await.unwrap();
                                    eprintln!("sending {} changes", changes.len());
                                    if !changes.is_empty() {
                                    msgs_tx_clone
                                        .send(serde_json::to_string(&changes).unwrap())
                                        .await
                                        .unwrap();

                                    }
                                } else if let Ok(changes) = serde_json::from_str::<
                                    Vec<automerge_protocol::UncompressedChange>,
                                >(text)
                                {
                                    eprintln!("received {} changes", changes.len());
                                    let changes: Vec<_> =
                                        changes.iter().map(Change::from).collect();
                                    if !changes.is_empty() {
                                        apply_changes_tx.send(changes).await.unwrap();
                                    }
                                } else {
                                    eprintln!("Unhandled message from client {:?}", text);
                                }
                            }
                        }
                    }
                })
            })
            .with(sync_log));
    tokio::join![doc_task, warp::serve(routes).run(([127, 0, 0, 1], 8080))];
}
