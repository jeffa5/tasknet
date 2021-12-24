use clap::Clap;
use options::Options;

mod auth;
mod backend;
mod options;
mod server;

#[tokio::main]
async fn main() {
    let options = Options::parse();

    server::run(options).await
}
