use std::{net::SocketAddr, path::PathBuf};

use clap::Clap;

#[derive(Clap)]
pub struct Options {
    #[clap(long, default_value = "127.0.0.1:8080")]
    pub http_listen_address: SocketAddr,
    #[clap(long, default_value = "127.0.0.1:8443")]
    pub https_listen_address: SocketAddr,
    #[clap(long, default_value = "certs/server.key")]
    pub key_file: PathBuf,
    #[clap(long, default_value = "certs/server.crt")]
    pub cert_file: PathBuf,
    #[clap(long, default_value = "tasknet-web/local")]
    pub static_files_dir: PathBuf,
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    #[clap(long, default_value = "username")]
    pub db_user: String,
    #[clap(long, default_value = "password", env = "DB_PASSWORD", setting = clap::ArgSettings::HideEnvValues)]
    pub db_password: String,
    #[clap(long, default_value = "tasknet")]
    pub db_name: String,
}
