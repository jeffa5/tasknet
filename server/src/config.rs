use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::auth::google::GoogleConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub serve_dir: PathBuf,
    pub documents_dir: PathBuf,

    pub google: Option<GoogleConfig>,
}

impl ServerConfig {
    pub fn load(file: &Path) -> Self {
        let mut bytes = Vec::new();
        let _ = File::open(file).and_then(|mut f| f.read_to_end(&mut bytes));
        serde_json::from_slice(&bytes).expect("Failed to read config file")
    }
}
