use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU16,
};

use serde::{Deserialize, Serialize};

// Based on https://taskwarrior.org/docs/design/task.html

pub fn now() -> DateTime {
    chrono::offset::Utc::now()
}

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    #[serde(flatten)]
    status: Status,
    #[serde(flatten)]
    core: Core,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Status {
    Pending(Pending),
    Deleted(Deleted),
    Completed(Completed),
    Waiting(Waiting),
    Recurring(Recurring),
}

impl Task {
    pub fn new() -> Self {
        Self {
            core: Core {
                uuid: uuid::Uuid::new_v4(),
                entry: now(),
                description: String::new(),
                project: Vec::new(),
                start: None,
                scheduled: None,
                notes: String::new(),
                tags: Vec::new(),
                priority: None,
                depends: HashSet::new(),
                udas: HashMap::new(),
            },
            status: Status::Pending(Pending { due: None }),
        }
    }

    pub const fn status(&self) -> &Status {
        &self.status
    }

    pub const fn entry(&self) -> &DateTime {
        &self.core.entry
    }

    pub const fn uuid(&self) -> uuid::Uuid {
        self.core.uuid
    }

    pub fn description(&self) -> &str {
        &self.core.description
    }

    pub fn set_description(&mut self, description: String) {
        self.core.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.core.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.core.project = project
    }

    pub const fn due(&self) -> &Option<DateTime> {
        match self.status {
            Status::Pending(ref t) => t.due(),
            Status::Deleted(ref t) => t.due(),
            Status::Completed(ref t) => t.due(),
            Status::Waiting(ref t) => t.due(),
            Status::Recurring(ref t) => t.due(),
        }
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        match self.status {
            Status::Pending(ref mut t) => t.set_due(due),
            Status::Deleted(ref mut t) => t.set_due(due),
            Status::Completed(ref mut t) => t.set_due(due),
            Status::Waiting(ref mut t) => t.set_due(due),
            Status::Recurring(ref mut t) => t.set_due(due),
        }
    }

    pub fn complete(&mut self) {
        match self.status {
            Status::Pending(ref t) => self.status = Status::Completed(t.complete()),
            Status::Deleted(_)
            | Status::Completed(_)
            | Status::Waiting(_)
            | Status::Recurring(_) => {}
        }
    }

    pub fn delete(&mut self) {
        match self.status {
            Status::Pending(ref t) => self.status = Status::Deleted(t.delete()),
            Status::Waiting(ref t) => self.status = Status::Deleted(t.delete()),
            Status::Recurring(_) | Status::Deleted(_) | Status::Completed(_) => {}
        }
    }

    pub fn restore(&mut self) {
        match self.status {
            Status::Pending(_) | Status::Waiting(_) | Status::Recurring(_) => {}
            Status::Completed(ref t) => self.status = Status::Pending(t.uncomplete()),
            Status::Deleted(ref t) => self.status = Status::Pending(t.undelete()),
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.core.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.core.tags = tags
    }

    pub const fn priority(&self) -> &Option<Priority> {
        &self.core.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.core.priority = priority
    }

    pub fn set_notes(&mut self, notes: String) {
        self.core.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.core.notes
    }

    pub const fn start(&self) -> &Option<DateTime> {
        &self.core.start
    }

    pub const fn end(&self) -> Option<&DateTime> {
        match self.status {
            Status::Pending(_) | Status::Waiting(_) | Status::Recurring(_) => None,
            Status::Completed(ref t) => Some(t.end()),
            Status::Deleted(ref t) => Some(t.end()),
        }
    }

    pub fn activate(&mut self) {
        self.core.tags.retain(|t| *t != "next");
        self.core.start = Some(now())
    }

    pub fn deactivate(&mut self) {
        self.core.start = None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pending {
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
}

impl Pending {
    pub fn delete(&self) -> Deleted {
        Deleted {
            end: now(),
            due: self.due,
        }
    }

    pub fn complete(&self) -> Completed {
        Completed {
            end: now(),
            due: self.due,
        }
    }

    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deleted {
    end: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
}

impl Deleted {
    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }

    pub const fn undelete(&self) -> Pending {
        Pending { due: self.due }
    }

    pub const fn end(&self) -> &DateTime {
        &self.end
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Completed {
    end: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
}

impl Completed {
    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }

    pub const fn uncomplete(&self) -> Pending {
        Pending { due: self.due }
    }

    pub const fn end(&self) -> &DateTime {
        &self.end
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waiting {
    wait: DateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
}

impl Waiting {
    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }

    pub fn delete(&self) -> Deleted {
        Deleted {
            end: now(),
            due: self.due,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recurring {
    recur: Recur,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
}

impl Recurring {
    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recur {
    amount: NonZeroU16,
    unit: RecurUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecurUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Core {
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled: Option<DateTime>,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    notes: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    project: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<Priority>,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    depends: HashSet<uuid::Uuid>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    udas: HashMap<String, UDA>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    #[serde(rename(serialize = "H", deserialize = "H"))]
    High,
    #[serde(rename(serialize = "M", deserialize = "M"))]
    Medium,
    #[serde(rename(serialize = "L", deserialize = "L"))]
    Low,
}

impl std::convert::TryFrom<String> for Priority {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_ref() {
            "h" => Ok(Self::High),
            "m" => Ok(Self::Medium),
            "l" => Ok(Self::Low),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UDA {
    Duration(String), // TODO: use custom newtype struct
    String(String),
    Number(f64),
    Date(DateTime),
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use regex::Regex;

    use super::*;

    #[test]
    fn test_serde_priority() {
        let mut task = Task::new();
        task.set_priority(Some(Priority::Medium));
        let s = serde_json::to_string(&task).unwrap();
        let t = serde_json::from_str(&s).unwrap();
        assert_eq!(task, t)
    }

    #[test]
    fn test_parse_json_format_string() {
        let json = r#"{"status":"completed","uuid":"2aa2717d-715f-4a74-9014-1ad4175bbbdc","entry":"2020-12-30T20:05:27.108Z","description":"Add uncomplete for completed tasks","end":"2020-12-31T11:10:42.123Z","project":["tasknet","ui"],"modified":"2020-12-31T11:10:42.123Z"}"#;
        let parsed = serde_json::from_str::<Task>(json);
        assert!(parsed.is_ok(), "{:?}", parsed)
    }

    #[test]
    fn test_render_json_format_string() {
        let task = Task::new();
        let rendered = serde_json::to_string(&task).unwrap();
        let re = Regex::new(r#"\{"status":"pending","uuid":".*","entry":".*","description":""}"#)
            .unwrap();
        assert!(re.is_match(&rendered), "{}", rendered)
    }
}
