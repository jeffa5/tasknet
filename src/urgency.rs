use crate::task::{Priority, Task};

const NEXT_COEFFICIENT: f64 = 15.0;
// urgency.due.coefficient                    12.0 # overdue or near due date
const DUE_COEFFICIENT: f64 = 12.0;
// urgency.blocking.coefficient                8.0 # blocking other tasks
const HIGH_PRIORITY_COEFFICIENT: f64 = 6.0;
const MEDIUM_PRIORITY_COEFFICIENT: f64 = 3.9;
const LOW_PRIORITY_COEFFICIENT: f64 = 1.8;
// urgency.scheduled.coefficient               5.0 # scheduled tasks
const ACTIVE_COEFFICIENT: f64 = 4.0;
const AGE_COEFFICIENT: f64 = 2.0;
// urgency.annotations.coefficient             1.0 # has annotations
const TAGS_COEFFICIENT: f64 = 1.0;
const PROJECT_COEFFICIENT: f64 = 1.0;
const WAITING_COEFFICIENT: f64 = -3.0;
// urgency.blocked.coefficient                 -5.0 # blocked by other tasks

// one week should be long enough for most tasks (for now)
const AGE_MAX_DAYS: f64 = 7.0;
const SECONDS_IN_A_DAY: f64 = 86400.0;

// https://github.com/GothenburgBitFactory/taskwarrior/blob/16529694eb0b06ed54331775e10bec32a72d01b1/src/Task.cpp#L1790
pub fn calculate(task: &Task) -> Option<f64> {
    match task {
        Task::Deleted(_) => None,
        Task::Completed(_) => None,
        Task::Waiting(task) => Some(
            WAITING_COEFFICIENT
                + urgency_age(*task.entry())
                + urgency_project(task.project())
                + urgency_due(task.due())
                + urgency_tags(task.tags())
                + urgency_next(task.tags())
                + urgency_priority(task.priority()),
        ),
        Task::Pending(task) => Some(
            urgency_age(*task.entry())
                + urgency_project(task.project())
                + urgency_active(task.start())
                + urgency_due(task.due())
                + urgency_tags(task.tags())
                + urgency_next(task.tags())
                + urgency_priority(task.priority()),
        ),
    }
}

fn urgency_age(entry: chrono::DateTime<chrono::Utc>) -> f64 {
    let days = (chrono::offset::Utc::now())
        .signed_duration_since(entry)
        .num_seconds() as f64;
    (days / (AGE_MAX_DAYS * SECONDS_IN_A_DAY)) * AGE_COEFFICIENT
}

fn urgency_project(project: &[String]) -> f64 {
    if project.is_empty() {
        0.0
    } else {
        PROJECT_COEFFICIENT
    }
}

fn urgency_active(start: &Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    if start.is_some() {
        ACTIVE_COEFFICIENT
    } else {
        0.0
    }
}

fn urgency_due(due: &Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    if let Some(due) = due {
        if due > &chrono::offset::Utc::now() {
            DUE_COEFFICIENT
        } else {
            0.0
        }
    } else {
        0.0
    }
}

fn urgency_tags(tags: &[String]) -> f64 {
    if tags.is_empty() {
        0.0
    } else {
        TAGS_COEFFICIENT
    }
}

fn urgency_next(tags: &[String]) -> f64 {
    if tags.contains(&"next".to_owned()) {
        NEXT_COEFFICIENT
    } else {
        0.0
    }
}

fn urgency_priority(priority: &Option<Priority>) -> f64 {
    match priority {
        None => 0.0,
        Some(Priority::Low) => LOW_PRIORITY_COEFFICIENT,
        Some(Priority::Medium) => MEDIUM_PRIORITY_COEFFICIENT,
        Some(Priority::High) => HIGH_PRIORITY_COEFFICIENT,
    }
}
