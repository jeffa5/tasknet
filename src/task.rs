use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub status: Status,
    pub uuid: uuid::Uuid,
    pub entry: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Deleted,
    Completed,
    Waiting,
    Recurring,
}

impl Task {
    pub fn new() -> Self {
        Task {
            status: Status::Pending,
            uuid: uuid::Uuid::new_v4(),
            entry: chrono::offset::Utc::now(),
            description: String::new(),
        }
    }
}
