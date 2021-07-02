use std::{collections::HashMap, ops::Deref, str::FromStr};

use automergeable::Automergeable;
use chrono::TimeZone;
#[allow(clippy::wildcard_imports)]
use serde::{Deserialize, Serialize};

// Based on https://taskwarrior.org/docs/design/task.html

pub fn now() -> DateTime {
    DateTime(chrono::Utc::now())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTime(pub chrono::DateTime<chrono::Utc>);

impl Default for DateTime {
    fn default() -> Self {
        now()
    }
}

impl Deref for DateTime {
    type Target = chrono::DateTime<chrono::Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl automergeable::ToAutomerge for DateTime {
    fn to_automerge(&self) -> automerge::Value {
        automerge::Value::Primitive(automerge::Primitive::Timestamp(self.timestamp()))
    }
}

impl automergeable::FromAutomerge for DateTime {
    fn from_automerge(
        value: &automerge::Value,
    ) -> std::result::Result<Self, automergeable::FromAutomergeError> {
        if let automerge::Value::Primitive(automerge::Primitive::Timestamp(i)) = value {
            let dt = chrono::Utc.timestamp(*i, 0);
            Ok(DateTime(dt))
        } else {
            Err(automergeable::FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a timestamp".to_owned(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Automergeable)]
pub struct Task {
    status: Status,
    entry: DateTime,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recur: Option<Recur>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    depends: Vec<Id>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    udas: HashMap<String, Uda>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Automergeable)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Deleted,
    Completed,
    Waiting,
    Recurring,
}

impl Default for Status {
    fn default() -> Self {
        Self::Pending
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}

impl Task {
    pub fn new() -> Self {
        Self {
            status: Status::Pending,
            entry: now(),
            description: String::new(),
            start: None,
            due: None,
            end: None,
            wait: None,
            scheduled: None,
            recur: None,
            parent: None,
            until: None,
            notes: String::new(),
            project: Vec::new(),
            tags: Vec::new(),
            priority: None,
            depends: Vec::new(),
            udas: HashMap::new(),
        }
    }

    pub const fn parent(&self) -> &Option<Id> {
        &self.parent
    }

    pub fn new_child(&self, uuid: uuid::Uuid) -> (uuid::Uuid, Self) {
        (
            uuid::Uuid::new_v4(),
            Self {
                entry: now(),
                description: self.description.clone(),
                project: self.project.clone(),
                start: self.start.clone(),
                scheduled: self.scheduled.clone(),
                notes: self.notes.clone(),
                tags: self.tags.clone(),
                priority: self.priority.clone(),
                depends: self.depends.clone(),
                udas: self.udas.clone(),
                status: Status::Pending,
                due: self.due.clone(),
                end: self.end.clone(),
                wait: self.wait.clone(),
                recur: self.recur.clone(),
                parent: Some(Id(uuid)),
                until: self.until.clone(),
            },
        )
    }

    pub const fn status(&self) -> &Status {
        &self.status
    }

    pub const fn entry(&self) -> &DateTime {
        &self.entry
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(&mut self, project: Vec<String>) {
        self.project = project
    }

    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(&mut self, due: Option<DateTime>) {
        self.due = due
    }

    pub const fn scheduled(&self) -> &Option<DateTime> {
        &self.scheduled
    }

    pub fn set_scheduled(&mut self, scheduled: Option<DateTime>) {
        self.scheduled = scheduled
    }

    pub const fn recur(&self) -> &Option<Recur> {
        &self.recur
    }

    pub fn set_recur(&mut self, recur: Option<Recur>) {
        match recur {
            None => {
                if self.status == Status::Recurring {
                    self.status = Status::Pending
                }
            }
            Some(_) => self.status = Status::Recurring,
        }
        self.recur = recur
    }

    pub fn complete(&mut self) {
        self.end = Some(now());
        self.start = None;
        self.status = Status::Completed;
    }

    pub fn delete(&mut self) {
        self.end = Some(now());
        self.status = Status::Deleted
    }

    pub fn restore(&mut self) {
        match self.status {
            Status::Pending | Status::Waiting | Status::Recurring => {}
            Status::Completed | Status::Deleted => self.status = Status::Pending,
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags
    }

    pub const fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: Option<Priority>) {
        self.priority = priority
    }

    pub fn set_notes(&mut self, notes: String) {
        self.notes = notes
    }

    pub fn notes(&self) -> &str {
        &self.notes
    }

    pub const fn start(&self) -> &Option<DateTime> {
        &self.start
    }

    pub const fn end(&self) -> &Option<DateTime> {
        match self.status {
            Status::Pending | Status::Waiting | Status::Recurring => &None,
            Status::Completed | Status::Deleted => &self.end,
        }
    }

    pub fn activate(&mut self) {
        let tags = self
            .tags
            .iter()
            .filter(|t| *t != "next")
            .map(std::borrow::ToOwned::to_owned)
            .collect();
        self.set_tags(tags);
        self.start = Some(now());
    }

    pub fn deactivate(&mut self) {
        self.start = None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Automergeable)]
pub struct Recur {
    pub amount: u16,
    pub unit: RecurUnit,
}

impl Recur {
    pub fn duration(&self) -> chrono::Duration {
        self.unit.duration() * i32::from(self.amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Automergeable)]
pub enum RecurUnit {
    Year,
    Month,
    Week,
    Day,
    Hour,
}

impl RecurUnit {
    fn duration(self) -> chrono::Duration {
        match self {
            Self::Year => chrono::Duration::weeks(52),
            Self::Month => chrono::Duration::weeks(4),
            Self::Week => chrono::Duration::weeks(1),
            Self::Day => chrono::Duration::days(1),
            Self::Hour => chrono::Duration::hours(1),
        }
    }
}

impl Default for RecurUnit {
    fn default() -> Self {
        Self::Week
    }
}

impl std::convert::TryFrom<String> for RecurUnit {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_ref() {
            "year" => Ok(Self::Year),
            "month" => Ok(Self::Month),
            "week" => Ok(Self::Week),
            "day" => Ok(Self::Day),
            "hour" => Ok(Self::Hour),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Automergeable)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Automergeable)]
#[serde(untagged)]
pub enum Uda {
    Duration(String), // TODO: use custom newtype struct
    String(String),
    Number(f64),
    Date(DateTime),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Id(pub uuid::Uuid);

impl ToString for Id {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for Id {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::from_str(s).map(Id)
    }
}

impl Deref for Id {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl automergeable::ToAutomerge for Id {
    fn to_automerge(&self) -> automerge::Value {
        automerge::Value::Primitive(automerge::Primitive::Str(self.to_string().into()))
    }
}

impl automergeable::FromAutomerge for Id {
    fn from_automerge(
        value: &automerge::Value,
    ) -> std::result::Result<Self, automergeable::FromAutomergeError> {
        if let automerge::Value::Primitive(automerge::Primitive::Str(s)) = value {
            Ok(Id(uuid::Uuid::parse_str(s).unwrap()))
        } else {
            Err(automergeable::FromAutomergeError::WrongType {
                found: value.clone(),
                expected: "a primitive string".to_owned(),
            })
        }
    }
}
