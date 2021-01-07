use serde::{Deserialize, Serialize};

use crate::task::{Priority, Status, Task};

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Filters {
    pub status_pending: bool,
    pub status_completed: bool,
    pub status_deleted: bool,
    pub status_waiting: bool,
    pub project: Vec<String>,
    pub tags: Vec<String>,
    pub description_and_notes: String,
    pub priority_none: bool,
    pub priority_low: bool,
    pub priority_medium: bool,
    pub priority_high: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            status_pending: true,
            status_completed: false,
            status_deleted: false,
            status_waiting: false,
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
            Status::Pending(_) => self.status_pending,
            Status::Deleted(_) => self.status_deleted,
            Status::Completed(_) => self.status_completed,
            Status::Waiting(_) => self.status_waiting,
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
