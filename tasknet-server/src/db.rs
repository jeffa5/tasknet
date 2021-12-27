use std::{fs, time::Duration};

use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use rand::Rng;
use tokio_postgres::{tls::MakeTlsConnect, Client, Config, NoTls, Socket};
use tracing::info;

use crate::options::Options;

pub async fn connect_to_db(options: &Options) -> tokio_postgres::Client {
    let mut postgres_config = tokio_postgres::Config::default();
    postgres_config
        .port(options.db_port)
        .host(&options.db_host)
        .dbname(&options.db_name)
        .user(&options.db_user)
        .password(&options.db_password);

    let postgres_client = if let Some(ca_path) = &options.db_ca_path {
        postgres_config.ssl_mode(tokio_postgres::config::SslMode::Require);
        let cert = fs::read(ca_path).expect("Failed to read certificate");
        let cert = Certificate::from_pem(&cert).expect("Failed to parse certificate as pem");
        let connector = TlsConnector::builder()
            .add_root_certificate(cert)
            .build()
            .expect("Failed to build tlsconnector");
        try_connect(postgres_config, MakeTlsConnector::new(connector), options).await
    } else {
        try_connect(postgres_config, NoTls, options).await
    };

    postgres_client
}

async fn try_connect<T>(config: Config, tls: T, options: &Options) -> Client
where
    T: MakeTlsConnect<Socket> + Clone,
    <T as MakeTlsConnect<Socket>>::Stream: Send + 'static,
{
    let mut backoff = 0;
    let retry_interval = 100;
    let (postgres_client, connection) = loop {
        match config.connect(tls.clone()).await {
            Ok(v) => {
                info!(
                    host = %options.db_host,
                    port = %options.db_port,
                    name = %options.db_name,
                    "Connected to DB"
                );
                break v;
            }
            Err(e) => {
                tracing::error!(error=%e, "Failed to connect to database");
                backoff += 1;
                backoff = std::cmp::min(backoff, 4);
                let duration_millis =
                    retry_interval * rand::thread_rng().gen_range(0..(2_u64.pow(backoff)));
                tokio::time::sleep(Duration::from_millis(duration_millis)).await
            }
        }
    };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!(error=?e, "connection closed");
        }
    });
    postgres_client
}
