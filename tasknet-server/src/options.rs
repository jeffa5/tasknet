use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
pub struct Options {
    #[clap(long, default_value = "127.0.0.1:8080")]
    pub http_listen_address: String,
    #[clap(long, default_value = "127.0.0.1:8443")]
    pub https_listen_address: String,
    #[clap(long, default_value = "certs/server.key")]
    pub key_file: PathBuf,
    #[clap(long, default_value = "certs/server.crt")]
    pub cert_file: PathBuf,
}
