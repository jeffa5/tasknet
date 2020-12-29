use crate::task::Task;

const NEXT_COEFFICIENT: f64 = 15.0;
// urgency.due.coefficient                    12.0 # overdue or near due date
// urgency.blocking.coefficient                8.0 # blocking other tasks
// urgency.uda.priority.H.coefficient          6.0 # high Priority
// urgency.uda.priority.M.coefficient          3.9 # medium Priority
// urgency.uda.priority.L.coefficient          1.8 # low Priority
// urgency.scheduled.coefficient               5.0 # scheduled tasks
// urgency.active.coefficient                  4.0 # already started tasks
const AGE_COEFFICIENT: f64 = 2.0;
// urgency.annotations.coefficient             1.0 # has annotations
// urgency.tags.coefficient                    1.0 # has tags
// urgency.project.coefficient                 1.0 # assigned to any project
// urgency.waiting.coefficient                 -3.0 # waiting task
// urgency.blocked.coefficient                 -5.0 # blocked by other tasks

// one week should be long enough for most tasks (for now)
const AGE_MAX_DAYS: f64 = 7.0;

// https://github.com/GothenburgBitFactory/taskwarrior/blob/16529694eb0b06ed54331775e10bec32a72d01b1/src/Task.cpp#L1790
pub fn calculate(task: &Task) -> f64 {
    let mut urgency = 0.0;
    urgency += urgency_age(task.entry) * AGE_COEFFICIENT;
    urgency
}

fn urgency_age(age: chrono::DateTime<chrono::Utc>) -> f64 {
    let days = age
        .signed_duration_since(chrono::offset::Utc::now())
        .num_days() as f64;
    days / AGE_MAX_DAYS
}
