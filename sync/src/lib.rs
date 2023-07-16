use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;

pub mod providers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    Message(Vec<u8>),
}

impl TryFrom<&Vec<u8>> for SyncMessage {
    type Error = serde_json::Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&[u8]> for SyncMessage {
    type Error = serde_json::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}

impl TryFrom<SyncMessage> for Vec<u8> {
    type Error = serde_json::Error;
    fn try_from(m: SyncMessage) -> Result<Self, Self::Error> {
        serde_json::to_vec(&m)
    }
}
