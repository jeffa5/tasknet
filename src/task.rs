use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

// Based on https://taskwarrior.org/docs/design/task.html

pub fn now() -> DateTime {
    chrono::offset::Utc::now()
}

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Task {
    Pending(PendingTask),
    Deleted(DeletedTask),
    Completed(CompletedTask),
    Waiting(WaitingTask),
    // Recurring(RecurringTask), // TODO
}

impl Task {
    pub fn new() -> Self {
        Task::Pending(PendingTask {
            uuid: uuid::Uuid::new_v4(),
            entry: chrono::offset::Utc::now(),
            description: String::new(),
            project: Vec::new(),
            start: None,
            due: None,
            until: None,
            scheduled: None,
            annotations: Vec::new(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTask {
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    annotations: Vec<Annotation>,
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

impl PendingTask {
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

    pub fn delete(mut self) -> DeletedTask {
        self.modified();
        DeletedTask {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            annotations: self.annotations,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }

    pub fn complete(mut self) -> CompletedTask {
        self.modified();
        CompletedTask {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            annotations: self.annotations,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled: Option<DateTime>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    annotations: Vec<Annotation>,
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

impl DeletedTask {
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

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn end(&self) -> &DateTime {
        &self.end
    }

    pub fn undelete(self) -> PendingTask {
        self.modified();
        PendingTask {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            start: None,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            annotations: self.annotations,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled: Option<DateTime>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    annotations: Vec<Annotation>,
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

impl CompletedTask {
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

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.modified();
        self.tags = tags
    }

    pub fn end(&self) -> &DateTime {
        &self.end
    }

    pub fn uncomplete(self) -> PendingTask {
        self.modified();
        PendingTask {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            start: None,
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            annotations: self.annotations,
            project: self.project,
            tags: self.tags,
            priority: self.priority,
            depends: self.depends,
            udas: self.udas,
            modified: self.modified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    wait: DateTime,
    // ---- optional ----
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled: Option<DateTime>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    annotations: Vec<Annotation>,
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

impl WaitingTask {
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

    pub fn delete(mut self) -> DeletedTask {
        self.modified();
        DeletedTask {
            uuid: self.uuid,
            entry: self.entry,
            description: self.description,
            end: now(),
            due: self.due,
            until: self.until,
            scheduled: self.scheduled,
            annotations: self.annotations,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Priority {
    #[serde(rename(serialize = "H", deserialize = "H"))]
    High,
    #[serde(rename(serialize = "M", deserialize = "M"))]
    Medium,
    #[serde(rename(serialize = "L", deserialize = "L"))]
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    entry: DateTime,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UDA {
    Duration(String), // TODO: use custom newtype struct
    String(String),
    Number(f64),
    Date(DateTime),
}
