use automerge::Path;
use automerge_protocol::ScalarValue;
use chrono::TimeZone;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use automerge::{LocalChange, Value};
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
    pub fn new(uuid: uuid::Uuid) -> Vec<automerge::LocalChange> {
        (Self {
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
            status: Status::Pending,
            due: None,
            end: None,
            wait: None,
            recur: None,
            parent: None,
            until: None,
        });
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
            LocalChange::set(
                task_path.clone().key("project"),
                Value::Sequence(Vec::new()),
            ),
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
        log!("set project");
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
        self.status = Status::Completed
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
        self.tags.retain(|t| *t != "next");
        self.start = Some(now())
    }

    pub fn deactivate(&mut self) {
        self.start = None
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
            let description = if let Some(Value::Text(t)) = map.get("description") {
                t.iter().collect()
            } else {
                return Err("Missing description / wrong type".to_owned());
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
            Ok(Self {
                status,
                entry,
                description,
                start: None,
                due: None,
                end: None,
                wait: None,
                scheduled: None,
                recur: None,
                parent: None,
                until: None,
                notes: "".to_owned(),
                project,
                tags: Vec::new(),
                priority: None,
                depends: HashSet::new(),
                udas: HashMap::new(),
            })
        } else {
            Err("Value was not a map".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use regex::Regex;

    use super::*;

    #[test]
    fn test_serde_priority() {
        let (_, mut task) = Task::new();
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
        let re = Regex::new(r#"\{"status":"pending","entry":".*","description":""}"#).unwrap();
        assert!(re.is_match(&rendered), "{}", rendered)
    }
}
