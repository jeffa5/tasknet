use crate::task::{Priority, Status, Task};

const NEXT_COEFFICIENT: f64 = 15.0;
const DUE_COEFFICIENT: f64 = 12.0;
// urgency.blocking.coefficient                8.0 # blocking other tasks
const HIGH_PRIORITY_COEFFICIENT: f64 = 6.0;
const MEDIUM_PRIORITY_COEFFICIENT: f64 = 3.9;
const LOW_PRIORITY_COEFFICIENT: f64 = 1.8;
const SCHEDULED_COEFFICIENT: f64 = 5.0;
const ACTIVE_COEFFICIENT: f64 = 4.0;
const AGE_COEFFICIENT: f64 = 2.0;
const TAGS_COEFFICIENT: f64 = 1.0;
const PROJECT_COEFFICIENT: f64 = 1.0;
const WAITING_COEFFICIENT: f64 = -3.0;
// urgency.blocked.coefficient                 -5.0 # blocked by other tasks

// one week should be long enough for most tasks (for now)
const AGE_MAX_DAYS: f64 = 7.0;
const SECONDS_IN_A_DAY: f64 = 86400.0;

// https://github.com/GothenburgBitFactory/taskwarrior/blob/16529694eb0b06ed54331775e10bec32a72d01b1/src/Task.cpp#L1790
pub fn calculate(task: &Task) -> Option<f64> {
    match task.status() {
        Status::Deleted | Status::Completed => None,
        Status::Waiting => Some(
            WAITING_COEFFICIENT
                + urgency_age(task.entry().0)
                + urgency_project(task.project())
                + urgency_due(&task.due().as_ref().map(|d| d.0))
                + urgency_scheduled(&task.scheduled().as_ref().map(|d| d.0))
                + urgency_tags(task.tags())
                + urgency_next(task.tags())
                + urgency_priority(task.priority()),
        ),
        Status::Pending => Some(
            urgency_age(task.entry().0)
                + urgency_project(task.project())
                + urgency_active(&task.start().as_ref().map(|d| d.0))
                + urgency_due(&task.due().as_ref().map(|d| d.0))
                + urgency_scheduled(&task.scheduled().as_ref().map(|d| d.0))
                + urgency_tags(task.tags())
                + urgency_next(task.tags())
                + urgency_priority(task.priority()),
        ),
    }
}

#[allow(clippy::cast_precision_loss)]
fn urgency_age(entry: chrono::DateTime<chrono::Utc>) -> f64 {
    let days = (chrono::offset::Utc::now())
        .signed_duration_since(entry)
        .num_seconds() as f64;
    (days / (AGE_MAX_DAYS * SECONDS_IN_A_DAY)) * AGE_COEFFICIENT
}

const fn urgency_project(project: &[String]) -> f64 {
    if project.is_empty() {
        0.0
    } else {
        PROJECT_COEFFICIENT
    }
}

const fn urgency_active(start: &Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    if start.is_some() {
        ACTIVE_COEFFICIENT
    } else {
        0.0
    }
}

#[allow(clippy::cast_precision_loss)]
fn urgency_due(due: &Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    due.map_or(0.0, |due| {
        let days_overdue = (chrono::offset::Utc::now())
            .signed_duration_since(due)
            .num_days();
        (if days_overdue > 7 {
            1.0
        } else if days_overdue >= -14 {
            ((days_overdue as f64 + 14.0) * 0.8 / 21.0) + 0.2
        } else {
            0.2
        }) * DUE_COEFFICIENT
    })
}

#[allow(clippy::cast_precision_loss)]
fn urgency_scheduled(scheduled: &Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    scheduled.map_or(0.0, |scheduled| {
        let days_overdue = (chrono::offset::Utc::now())
            .signed_duration_since(scheduled)
            .num_days();
        (if days_overdue > 7 {
            1.0
        } else if days_overdue >= -14 {
            ((days_overdue as f64 + 14.0) * 0.8 / 21.0) + 0.2
        } else {
            0.2
        }) * SCHEDULED_COEFFICIENT
    })
}

fn urgency_tags(tags: &[String]) -> f64 {
    (match tags.len() {
        0 => 0.0,
        1 => 0.8,
        2 => 0.9,
        _ => 1.0,
    }) * TAGS_COEFFICIENT
}

fn urgency_next(tags: &[String]) -> f64 {
    if tags.contains(&"next".to_owned()) {
        NEXT_COEFFICIENT
    } else {
        0.0
    }
}

const fn urgency_priority(priority: &Option<Priority>) -> f64 {
    match priority {
        None => 0.0,
        Some(Priority::Low) => LOW_PRIORITY_COEFFICIENT,
        Some(Priority::Medium) => MEDIUM_PRIORITY_COEFFICIENT,
        Some(Priority::High) => HIGH_PRIORITY_COEFFICIENT,
    }
}
