use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderDefault {
    pub enabled: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Providers {
    pub public: ProviderDefault,
    pub google: ProviderDefault,
}
