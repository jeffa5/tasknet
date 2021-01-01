use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

// Based on https://taskwarrior.org/docs/design/task.html

pub fn now() -> DateTime {
    chrono::offset::Utc::now()
}

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Task {
    Pending(Pending),
    Deleted(Deleted),
    Completed(Completed),
    Waiting(Waiting),
    // Recurring(Recurring), // TODO
}

impl Task {
    pub fn new() -> Self {
        Task::Pending(Pending {
            uuid: uuid::Uuid::new_v4(),
            entry: chrono::offset::Utc::now(),
            description: String::new(),
            project: Vec::new(),
            start: None,
            due: None,
            until: None,
            scheduled: None,
            notes: String::new(),
            tags: Vec::new(),
            priority: None,
            depends: HashSet::new(),
            udas: HashMap::new(),
            modified: chrono::offset::Utc::now(),
        })
    }

    pub fn entry(&self) -> &DateTime {
        match self {
            Self::Pending(t) => t.entry(),
            Self::Deleted(t) => t.entry(),
            Self::Completed(t) => t.entry(),
            Self::Waiting(t) => t.entry(),
        }
    }

    pub fn uuid(&self) -> uuid::Uuid {
        match self {
            Self::Pending(t) => t.uuid(),
            Self::Deleted(t) => t.uuid(),
            Self::Completed(t) => t.uuid(),
            Self::Waiting(t) => t.uuid(),
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Pending(t) => t.description(),
            Self::Deleted(t) => t.description(),
            Self::Completed(t) => t.description(),
            Self::Waiting(t) => t.description(),
        }
    }

    pub fn set_description(&mut self, description: String) {
        match self {
            Self::Pending(t) => t.set_description(description),
            Self::Deleted(t) => t.set_description(description),
            Self::Completed(t) => t.set_description(description),
            Self::Waiting(t) => t.set_description(description),
        }
    }

    pub fn project(&self) -> &[String] {
        match self {
            Self::Pending(t) => t.project(),
            Self::Deleted(t) => t.project(),
            Self::Completed(t) => t.project(),
            Self::Waiting(t) => t.project(),
        }
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        match self {
            Self::Pending(t) => t.set_project(project),
            Self::Deleted(t) => t.set_project(project),
            Self::Completed(t) => t.set_project(project),
            Self::Waiting(t) => t.set_project(project),
        }
    }

    pub fn due(&self) -> &Option<DateTime> {
        match self {
            Self::Pending(t) => t.due(),
            Self::Deleted(t) => t.due(),
            Self::Completed(t) => t.due(),
            Self::Waiting(t) => t.due(),
        }
    }

    pub fn complete(self) -> Self {
        match self {
            Self::Pending(t) => Self::Completed(t.complete()),
            Self::Deleted(_) => self,
            Self::Completed(_) => self,
            Self::Waiting(_) => self,
        }
    }

    pub fn delete(self) -> Self {
        match self {
            Self::Pending(t) => Self::Deleted(t.delete()),
            Self::Deleted(_) => self,
            Self::Completed(_) => self,
            Self::Waiting(t) => Self::Deleted(t.delete()),
        }
    }

    pub fn tags(&self) -> &[String] {
        match self {
            Self::Pending(t) => t.tags(),
            Self::Deleted(t) => t.tags(),
            Self::Completed(t) => t.tags(),
            Self::Waiting(t) => t.tags(),
        }
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        match self {
            Self::Pending(t) => t.set_tags(tags),
            Self::Deleted(t) => t.set_tags(tags),
            Self::Completed(t) => t.set_tags(tags),
            Self::Waiting(t) => t.set_tags(tags),
        }
    }

    pub fn priority(&self) -> &Option<Priority> {
        match self {
            Self::Pending(t) => t.priority(),
            Self::Deleted(t) => t.priority(),
            Self::Completed(t) => t.priority(),
            Self::Waiting(t) => t.priority(),
        }
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        match self {
            Self::Pending(t) => t.set_priority(priority),
            Self::Deleted(t) => t.set_priority(priority),
            Self::Completed(t) => t.set_priority(priority),
            Self::Waiting(t) => t.set_priority(priority),
        }
    }

    pub fn set_notes(&mut self, notes: String) {
        match self {
            Self::Pending(t) => t.set_notes(notes),
            Self::Deleted(t) => t.set_notes(notes),
            Self::Completed(t) => t.set_notes(notes),
            Self::Waiting(t) => t.set_notes(notes),
        }
    }

    pub fn notes(&self) -> &str {
        match self {
            Self::Pending(t) => t.notes(),
            Self::Deleted(t) => t.notes(),
            Self::Completed(t) => t.notes(),
            Self::Waiting(t) => t.notes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pending {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
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
    #[serde(flatten)]
    udas: HashMap<String, UDA>,
    // ---- internal ----
    modified: DateTime,
}

impl Pending {
    fn modified(&mut self) {
        self.modified = chrono::offset::Utc::now();
    }

    pub fn entry(&self) -> &DateTime {
        &self.entry
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.modified();
        self.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.modified();
        self.project = project
    }

    pub fn delete(mut self) -> Deleted {
        self.modified();
        Deleted {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            start: self.start,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            notes: self.notes,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }

    pub fn complete(mut self) -> Completed {
        self.modified();
        Completed {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            start: self.start,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            notes: self.notes,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }

    pub fn start(&self) -> &Option<DateTime> {
        &self.start
    }

    pub fn activate(&mut self) {
        self.modified();
        self.tags.retain(|t| *t != "next");
        self.start = Some(now())
    }

    pub fn deactivate(&mut self) {
        self.modified();
        self.start = None
    }

    pub fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.modified();
        self.priority = priority
    }

    pub fn set_notes(&mut self, notes: String) {
        self.modified();
        self.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deleted {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
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
    #[serde(flatten)]
    udas: HashMap<String, UDA>,
    // ---- internal ----
    modified: DateTime,
}

impl Deleted {
    fn modified(&mut self) {
        self.modified = chrono::offset::Utc::now();
    }

    pub fn entry(&self) -> &DateTime {
        &self.entry
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.modified();
        self.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.modified();
        self.project = project
    }

    pub fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.modified();
        self.priority = priority
    }

    pub fn end(&self) -> &DateTime {
        &self.end
    }

    pub fn set_notes(&mut self, notes: String) {
        self.modified();
        self.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub fn undelete(mut self) -> Pending {
        self.modified();
        Pending {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            start: None,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            notes: self.notes,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Completed {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
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
    #[serde(flatten)]
    udas: HashMap<String, UDA>,
    // ---- internal ----
    modified: DateTime,
}

impl Completed {
    fn modified(&mut self) {
        self.modified = chrono::offset::Utc::now();
    }

    pub fn entry(&self) -> &DateTime {
        &self.entry
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.modified();
        self.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.modified();
        self.project = project
    }

    pub fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.modified();
        self.priority = priority
    }

    pub fn end(&self) -> &DateTime {
        &self.end
    }

    pub fn set_notes(&mut self, notes: String) {
        self.modified();
        self.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub fn uncomplete(mut self) -> Pending {
        self.modified();
        Pending {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            start: None,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            notes: self.notes,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Waiting {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    wait: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
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
    #[serde(flatten)]
    udas: HashMap<String, UDA>,
    // ---- internal ----
    modified: DateTime,
}

impl Waiting {
    fn modified(&mut self) {
        self.modified = chrono::offset::Utc::now();
    }

    pub fn entry(&self) -> &DateTime {
        &self.entry
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.modified();
        self.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.modified();
        self.project = project
    }

    pub fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn delete(mut self) -> Deleted {
        self.modified();
        Deleted {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            start: self.start,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            notes: self.notes,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.modified();
        self.priority = priority
    }

    pub fn set_notes(&mut self, notes: String) {
        self.modified();
        self.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }
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
            "h" => Ok(Priority::High),
            "m" => Ok(Priority::Medium),
            "l" => Ok(Priority::Low),
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

    use super::*;

    #[test]
    fn test_serde_priority() {
        let mut task = Task::new();
        task.set_priority(Some(Priority::Medium));
        let s = serde_json::to_string(&task).unwrap();
        let t = serde_json::from_str(&s).unwrap();
        assert_eq!(task, t)
    }
}
