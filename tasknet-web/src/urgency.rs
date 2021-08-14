use crate::{
    settings::UrgencySettings,
    task::{DateTime, Priority, Status, Task},
};

// urgency.blocking.coefficient                8.0 # blocking other tasks
// urgency.blocked.coefficient                 -5.0 # blocked by other tasks

// one week should be long enough for most tasks (for now)
const AGE_MAX_DAYS: f64 = 7.0;
const SECONDS_IN_A_DAY: f64 = 86400.0;

impl UrgencySettings {
    // https://github.com/GothenburgBitFactory/taskwarrior/blob/16529694eb0b06ed54331775e10bec32a72d01b1/src/Task.cpp#L1790
    pub fn calculate(&self, task: &Task) -> Option<f64> {
        match task.status() {
            Status::Deleted | Status::Completed | Status::Recurring => None,
            Status::Waiting => Some(
                self.waiting
                    + self.urgency_age(task.entry())
                    + self.urgency_project(task.project())
                    + self.urgency_due(task.due())
                    + self.urgency_scheduled(task.scheduled())
                    + self.urgency_tags(task.tags())
                    + self.urgency_next(task.tags())
                    + self.urgency_priority(task.priority())
                    + self.urgency_notes(task.notes()),
            ),
            Status::Pending => Some(
                self.urgency_age(task.entry())
                    + self.urgency_project(task.project())
                    + self.urgency_active(task.start())
                    + self.urgency_due(task.due())
                    + self.urgency_scheduled(task.scheduled())
                    + self.urgency_tags(task.tags())
                    + self.urgency_next(task.tags())
                    + self.urgency_priority(task.priority())
                    + self.urgency_notes(task.notes()),
            ),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn urgency_age(&self, entry: &DateTime) -> f64 {
        let days = (chrono::offset::Utc::now())
            .signed_duration_since(**entry)
            .num_seconds() as f64;
        (days / (AGE_MAX_DAYS * SECONDS_IN_A_DAY)) * self.age
    }

    const fn urgency_project(&self, project: &[String]) -> f64 {
        if project.is_empty() {
            0.0
        } else {
            self.project
        }
    }

    const fn urgency_active(&self, start: &Option<DateTime>) -> f64 {
        if start.is_some() {
            self.active
        } else {
            0.0
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn urgency_due(&self, due: &Option<DateTime>) -> f64 {
        due.as_ref().map_or(0.0, |due| {
            let days_overdue = (chrono::offset::Utc::now())
                .signed_duration_since(**due)
                .num_days();
            (if days_overdue > 7 {
                1.0
            } else if days_overdue >= -14 {
                ((days_overdue as f64 + 14.0) * 0.8 / 21.0) + 0.2
            } else {
                0.2
            }) * self.due
        })
    }

    #[allow(clippy::cast_precision_loss)]
    fn urgency_scheduled(&self, scheduled: &Option<DateTime>) -> f64 {
        scheduled.as_ref().map_or(0.0, |scheduled| {
            let days_overdue = (chrono::offset::Utc::now())
                .signed_duration_since(**scheduled)
                .num_days();
            (if days_overdue > 7 {
                1.0
            } else if days_overdue >= -14 {
                ((days_overdue as f64 + 14.0) * 0.8 / 21.0) + 0.2
            } else {
                0.2
            }) * self.scheduled
        })
    }

    fn urgency_tags(&self, tags: &[String]) -> f64 {
        (match tags.len() {
            0 => 0.0,
            1 => 0.8,
            2 => 0.9,
            _ => 1.0,
        }) * self.tags
    }

    fn urgency_next(&self, tags: &[String]) -> f64 {
        if tags.contains(&"next".to_owned()) {
            self.next
        } else {
            0.0
        }
    }

    const fn urgency_priority(&self, priority: &Option<Priority>) -> f64 {
        match priority {
            None => 0.0,
            Some(Priority::Low) => self.low_priority,
            Some(Priority::Medium) => self.medium_priority,
            Some(Priority::High) => self.high_priority,
        }
    }

    const fn urgency_notes(&self, notes: &str) -> f64 {
        if notes.is_empty() {
            0.0
        } else {
            self.notes
        }
    }
}
