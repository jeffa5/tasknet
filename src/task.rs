use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use automerge::{LocalChange, Path, Value};
use automerge_protocol::ScalarValue;
use chrono::TimeZone;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};

// Based on https://taskwarrior.org/docs/design/task.html

pub fn now() -> DateTime {
    chrono::offset::Utc::now()
}

type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    parent: Option<uuid::Uuid>,
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
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    #[serde(default)]
    depends: HashSet<uuid::Uuid>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    udas: HashMap<String, UDA>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Deleted,
    Completed,
    Waiting,
    Recurring,
}

impl Task {
    pub fn create(uuid: uuid::Uuid) -> Vec<automerge::LocalChange> {
        let task_path = Path::root().key("tasks").key(uuid.to_string());
        vec![
            LocalChange::set(
                task_path.clone(),
                Value::Map(HashMap::new(), automerge_protocol::MapType::Map),
            ),
            LocalChange::set(
                task_path.clone().key("entry"),
                Value::Primitive(ScalarValue::Timestamp(now().timestamp_millis())),
            ),
            LocalChange::set(
                task_path.clone().key("description"),
                Value::Text(Vec::new()),
            ),
            LocalChange::set(task_path.clone().key("notes"), Value::Text(Vec::new())),
            LocalChange::set(
                task_path.clone().key("project"),
                Value::Sequence(Vec::new()),
            ),
            LocalChange::set(task_path.clone().key("tags"), Value::Sequence(Vec::new())),
            LocalChange::set(
                task_path.clone().key("start"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("status"),
                Value::Primitive(ScalarValue::Str(
                    serde_json::to_string(&Status::Pending).unwrap(),
                )),
            ),
            LocalChange::set(
                task_path.clone().key("entry"),
                Value::Primitive(ScalarValue::Timestamp(now().timestamp_millis())),
            ),
            LocalChange::set(
                task_path.clone().key("end"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("start"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("wait"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("until"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("scheduled"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.clone().key("due"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                task_path.key("priority"),
                Value::Primitive(ScalarValue::Null),
            ),
        ]
    }

    pub const fn parent(&self) -> &Option<uuid::Uuid> {
        &self.parent
    }

    pub fn new_child(&self, uuid: uuid::Uuid) -> (uuid::Uuid, Self) {
        (
            uuid::Uuid::new_v4(),
            Self {
                entry: now(),
                description: self.description.clone(),
                project: self.project.clone(),
                start: self.start,
                scheduled: self.scheduled,
                notes: self.notes.clone(),
                tags: self.tags.clone(),
                priority: self.priority.clone(),
                depends: self.depends.clone(),
                udas: self.udas.clone(),
                status: Status::Pending,
                due: self.due,
                end: self.end,
                wait: self.wait,
                recur: self.recur.clone(),
                parent: Some(uuid),
                until: self.until,
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

    pub fn set_description(path: Path, description: &str) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("description"),
            Value::Text(description.chars().collect()),
        )]
    }

    pub fn project(&self) -> &[String] {
        &self.project
    }

    pub fn set_project(path: Path, project: Vec<String>) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("project"),
            Value::Sequence(
                project
                    .into_iter()
                    .map(|s| Value::Text(s.chars().collect()))
                    .collect(),
            ),
        )]
    }

    pub const fn due(&self) -> &Option<DateTime> {
        &self.due
    }

    pub fn set_due(path: Path, due: Option<DateTime>) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("due"),
            Value::Primitive(due.map_or(ScalarValue::Null, |due| {
                ScalarValue::Timestamp(due.timestamp_millis())
            })),
        )]
    }

    pub const fn scheduled(&self) -> &Option<DateTime> {
        &self.scheduled
    }

    pub fn set_scheduled(path: Path, scheduled: Option<DateTime>) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("scheduled"),
            Value::Primitive(scheduled.map_or(ScalarValue::Null, |scheduled| {
                ScalarValue::Timestamp(scheduled.timestamp_millis())
            })),
        )]
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

    pub fn complete(path: Path) -> Vec<LocalChange> {
        vec![
            LocalChange::set(
                path.clone().key("end"),
                Value::Primitive(ScalarValue::Timestamp(now().timestamp_millis())),
            ),
            LocalChange::set(
                path.clone().key("start"),
                Value::Primitive(ScalarValue::Null),
            ),
            LocalChange::set(
                path.key("status"),
                Value::Primitive(ScalarValue::Str(
                    serde_json::to_string(&Status::Completed).unwrap(),
                )),
            ),
        ]
    }

    pub fn delete(path: Path) -> Vec<LocalChange> {
        vec![
            LocalChange::set(
                path.clone().key("end"),
                Value::Primitive(ScalarValue::Timestamp(now().timestamp_millis())),
            ),
            LocalChange::set(
                path.key("status"),
                Value::Primitive(ScalarValue::Str(
                    serde_json::to_string(&Status::Deleted).unwrap(),
                )),
            ),
        ]
    }

    pub fn restore(&self, path: Path) -> Vec<LocalChange> {
        match self.status {
            Status::Pending | Status::Waiting | Status::Recurring => Vec::new(),
            Status::Completed | Status::Deleted => vec![LocalChange::set(
                path.key("status"),
                Value::Primitive(ScalarValue::Str(
                    serde_json::to_string(&Status::Pending).unwrap(),
                )),
            )],
        }
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn set_tags(path: Path, tags: Vec<String>) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("tags"),
            Value::Sequence(
                tags.into_iter()
                    .map(|s| Value::Text(s.chars().collect()))
                    .collect(),
            ),
        )]
    }

    pub const fn priority(&self) -> &Option<Priority> {
        &self.priority
    }

    pub fn set_priority(path: Path, priority: Option<Priority>) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("priority"),
            Value::Primitive(priority.map_or(ScalarValue::Null, |priority| {
                ScalarValue::Str(serde_json::to_string(&priority).unwrap())
            })),
        )]
    }

    pub fn set_notes(path: Path, notes: &str) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("notes"),
            Value::Text(notes.chars().collect()),
        )]
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

    pub fn activate(&self, path: Path) -> Vec<LocalChange> {
        let tags = self
            .tags
            .iter()
            .filter(|t| *t != "next")
            .map(|t| t.to_owned())
            .collect();
        let mut changes = Self::set_tags(path.clone(), tags);
        changes.push(LocalChange::set(
            path.key("start"),
            Value::Primitive(ScalarValue::Timestamp(now().timestamp_millis())),
        ));
        changes
    }

    pub fn deactivate(path: Path) -> Vec<LocalChange> {
        vec![LocalChange::set(
            path.key("start"),
            Value::Primitive(ScalarValue::Null),
        )]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recur {
    pub amount: u16,
    pub unit: RecurUnit,
}

impl Recur {
    pub fn duration(&self) -> chrono::Duration {
        self.unit.duration() * i32::from(self.amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl TryFrom<automerge::Value> for Task {
    type Error = String;

    fn try_from(value: automerge::Value) -> Result<Self, Self::Error> {
        if let automerge::Value::Map(map, automerge_protocol::MapType::Map) = value {
            let status =
                if let Some(Value::Primitive(automerge::ScalarValue::Str(s))) = map.get("status") {
                    serde_json::from_str(s).unwrap()
                } else {
                    return Err("Missing status / wrong type".to_owned());
                };
            let entry = if let Some(Value::Primitive(automerge::ScalarValue::Timestamp(t))) =
                map.get("entry")
            {
                chrono::Utc.timestamp_millis(*t)
            } else {
                return Err("Missing entry / wrong type".to_owned());
            };
            let start = match map.get("start") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing start / wrong type".to_owned()),
            };
            let end = match map.get("end") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing end / wrong type".to_owned()),
            };
            let wait = match map.get("wait") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing wait / wrong type".to_owned()),
            };
            let until = match map.get("until") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing until / wrong type".to_owned()),
            };
            let scheduled = match map.get("scheduled") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing scheduled / wrong type".to_owned()),
            };
            let due = match map.get("due") {
                Some(Value::Primitive(ScalarValue::Timestamp(t))) => {
                    Some(chrono::Utc.timestamp_millis(*t))
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing due / wrong type".to_owned()),
            };
            let description = if let Some(Value::Text(t)) = map.get("description") {
                t.iter().collect()
            } else {
                return Err("Missing description / wrong type".to_owned());
            };
            let notes = if let Some(Value::Text(t)) = map.get("notes") {
                t.iter().collect()
            } else {
                return Err("Missing notes / wrong type".to_owned());
            };
            let project = if let Some(Value::Sequence(v)) = map.get("project") {
                let mut project: Vec<String> = Vec::new();
                for i in v {
                    if let Value::Text(t) = i {
                        project.push(t.iter().collect())
                    } else {
                        return Err(format!("Wrong type in project sequence {:?}", i));
                    }
                }
                project
            } else {
                return Err("Missing project / wrong type".to_owned());
            };
            let tags = if let Some(Value::Sequence(v)) = map.get("tags") {
                let mut tags: Vec<String> = Vec::new();
                for i in v {
                    if let Value::Text(t) = i {
                        tags.push(t.iter().collect())
                    } else {
                        return Err(format!("Wrong type in tags sequence {:?}", i));
                    }
                }
                tags
            } else {
                return Err("Missing tags / wrong type".to_owned());
            };
            let priority = match map.get("priority") {
                Some(Value::Primitive(ScalarValue::Str(s))) => {
                    Some(serde_json::from_str(s).unwrap())
                }
                Some(Value::Primitive(ScalarValue::Null)) => None,
                _ => return Err("Missing priority / wrong type".to_owned()),
            };
            Ok(Self {
                status,
                entry,
                description,
                start,
                due,
                end,
                wait,
                scheduled,
                recur: None,
                parent: None,
                until,
                notes,
                project,
                tags,
                priority,
                depends: HashSet::new(),
                udas: HashMap::new(),
            })
        } else {
            Err("Value was not a map".to_owned())
        }
    }
}
