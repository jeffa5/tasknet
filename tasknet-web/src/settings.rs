use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub document_id: uuid::Uuid,
    pub urgency: UrgencySettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UrgencySettings {
    pub next: f64,
    pub due: f64,
    pub high_priority: f64,
    pub medium_priority: f64,
    pub low_priority: f64,
    pub scheduled: f64,
    pub active: f64,
    pub age: f64,
    pub notes: f64,
    pub tags: f64,
    pub project: f64,
    pub waiting: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            document_id: uuid::Uuid::new_v4(),
            urgency: UrgencySettings::default(),
        }
    }
}

impl Default for UrgencySettings {
    fn default() -> Self {
        Self {
            next: 15.0,
            due: 12.0,
            high_priority: 6.0,
            medium_priority: 4.0,
            low_priority: 2.0,
            scheduled: 5.0,
            active: 4.0,
            age: 2.0,
            notes: 1.0,
            tags: 1.0,
            project: 1.0,
            waiting: -3.0,
        }
    }
}
