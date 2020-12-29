use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

// Based on https://taskwarrior.org/docs/design/task.html

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
            annotation: Vec::new(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    // ---- optional ----
    start: Option<DateTime>,
    due: Option<DateTime>,
    until: Option<DateTime>,
    scheduled: Option<DateTime>,
    annotation: Vec<Annotation>,
    project: Vec<String>,
    tags: Vec<String>,
    priority: Option<Priority>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    start: Option<DateTime>,
    due: Option<DateTime>,
    until: Option<DateTime>,
    scheduled: Option<DateTime>,
    annotation: Vec<Annotation>,
    project: Vec<String>,
    tags: Vec<String>,
    priority: Option<Priority>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    end: DateTime,
    // ---- optional ----
    start: Option<DateTime>,
    due: Option<DateTime>,
    until: Option<DateTime>,
    scheduled: Option<DateTime>,
    annotation: Vec<Annotation>,
    project: Vec<String>,
    tags: Vec<String>,
    priority: Option<Priority>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingTask {
    // ---- required ----
    uuid: uuid::Uuid,
    entry: DateTime,
    description: String,
    wait: DateTime,
    // ---- optional ----
    start: Option<DateTime>,
    due: Option<DateTime>,
    until: Option<DateTime>,
    scheduled: Option<DateTime>,
    annotation: Vec<Annotation>,
    project: Vec<String>,
    tags: Vec<String>,
    priority: Option<Priority>,
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
