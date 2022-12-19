use std::{net::SocketAddr, path::PathBuf};

use axum::Router;
use axum_extra::routing::SpaRouter;
use clap::Parser;

#[derive(clap::Parser)]
struct ServerOptions {
    #[clap(long, short, default_value = "3000")]
    port: u16,
    #[clap(long, short, default_value = "web/dist")]
    serve_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let options = ServerOptions::parse();

    let app = Router::new().merge(SpaRouter::new("/", options.serve_dir).index_file("index.html"));

    let addr = SocketAddr::from(([127, 0, 0, 1], options.port));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
