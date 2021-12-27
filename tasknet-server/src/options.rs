use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
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
    #[clap(long, default_value = "localhost", env)]
    pub db_host: String,
    #[clap(long, default_value = "5432", env)]
    pub db_port: u16,
    #[clap(long, default_value = "username", env)]
    pub db_user: String,
    #[clap(long, default_value = "password", env)]
    pub db_password: String,
    #[clap(long, default_value = "tasknet", env)]
    pub db_name: String,
    #[clap(long, env)]
    pub db_ca_path: PathBuf,
    #[clap(long, default_value = "http://kratos:4433/")]
    pub kratos_url: String,
}
