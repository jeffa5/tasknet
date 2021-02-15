use futures_util::{FutureExt, StreamExt};
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();
    let tasknet_log = warp::log("tasknet::tasknet");
    let sync_log = warp::log("tasknet::sync");
    let routes = warp::get()
        .and(
            warp::path("tasknet").and(warp::fs::dir("tasknet-web/local/tasknet").with(tasknet_log)),
        )
        .or(warp::path("sync")
            .and(warp::ws())
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|websocket| {
                    eprintln!("websocket connection");
                    // Just echo all messages back...
                    let (tx, rx) = websocket.split();
                    rx.forward(tx).map(|result| {
                        if let Err(e) = result {
                            eprintln!("websocket error: {:?}", e);
                        }
                    })
                })
            })
            .with(sync_log));
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
