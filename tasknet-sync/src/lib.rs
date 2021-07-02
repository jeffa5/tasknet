use std::convert::{TryFrom, TryInto};

use automerge_backend::SyncMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Message {
    SyncMessage(automerge_backend::SyncMessage),
}

impl From<EncodedMessage> for Message {
    fn from(encoded: EncodedMessage) -> Self {
        match encoded {
            EncodedMessage::SyncMessage(sync_bytes) => {
                Self::SyncMessage(SyncMessage::decode(&sync_bytes).unwrap())
            }
        }
    }
}

impl From<Message> for EncodedMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::SyncMessage(sync_message) => {
                let bytes = sync_message.encode().unwrap();
                Self::SyncMessage(bytes)
            }
        }
    }
}

impl From<Vec<u8>> for Message {
    fn from(bytes: Vec<u8>) -> Self {
        let encoded_message = EncodedMessage::try_from(bytes).unwrap();
        Message::from(encoded_message)
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        let encoded = EncodedMessage::from(message);
        encoded.try_into().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EncodedMessage {
    SyncMessage(Vec<u8>),
}

impl TryFrom<EncodedMessage> for Vec<u8> {
    type Error = serde_json::Error;

    fn try_from(value: EncodedMessage) -> Result<Self, Self::Error> {
        serde_json::to_vec(&value)
    }
}

impl TryFrom<&[u8]> for EncodedMessage {
    type Error = serde_json::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(value)
    }
}

impl TryFrom<Vec<u8>> for EncodedMessage {
    type Error = serde_json::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}
