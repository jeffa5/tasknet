use serde::{Deserialize, Serialize};

use crate::task::{Priority, Status, Task};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Filters {
    #[serde(default)]
    pub status_pending: bool,
    #[serde(default)]
    pub status_completed: bool,
    #[serde(default)]
    pub status_deleted: bool,
    #[serde(default)]
    pub status_waiting: bool,
    #[serde(default)]
    pub status_recurring: bool,
    #[serde(default)]
    pub project: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description_and_notes: String,
    #[serde(default)]
    pub priority_none: bool,
    #[serde(default)]
    pub priority_low: bool,
    #[serde(default)]
    pub priority_medium: bool,
    #[serde(default)]
    pub priority_high: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            status_pending: true,
            status_completed: false,
            status_deleted: false,
            status_waiting: false,
            status_recurring: false,
            project: Vec::new(),
            tags: Vec::new(),
            description_and_notes: String::new(),
            priority_none: true,
            priority_low: true,
            priority_medium: true,
            priority_high: true,
        }
    }
}

impl Filters {
    pub fn filter_task(&self, task: &Task) -> bool {
        let filter_status = match task.status() {
            Status::Pending => self.status_pending,
            Status::Deleted => self.status_deleted,
            Status::Completed => self.status_completed,
            Status::Waiting => self.status_waiting,
            Status::Recurring => self.status_recurring,
        };
        let filter_project = task
            .project()
            .join(".")
            .to_lowercase()
            .contains(&self.project.join(".").to_lowercase());
        let filter_tags = self.tags.iter().all(|tag| {
            task.tags()
                .iter()
                .any(|t| t.to_lowercase().contains(&tag.to_lowercase()))
        });
        let filter_description_and_notes = task
            .description()
            .to_lowercase()
            .contains(&self.description_and_notes.to_lowercase())
            || task
                .notes()
                .to_lowercase()
                .contains(&self.description_and_notes.to_lowercase());
        let filter_priority = match task.priority() {
            None => self.priority_none,
            Some(Priority::Low) => self.priority_low,
            Some(Priority::Medium) => self.priority_medium,
            Some(Priority::High) => self.priority_high,
        };
        filter_status
            && filter_project
            && filter_tags
            && filter_description_and_notes
            && filter_priority
    }
}
