use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub status: Status,
    pub uuid: uuid::Uuid,
    pub entry: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Deleted,
    Completed,
    Waiting,
    Recurring,
}
