use std::{collections::HashMap, convert::TryFrom, str::FromStr};

use automerge::{Backend, Frontend, Path};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{task::Task, Msg};

pub const TASKS_STORAGE_KEY: &str = "tasknet-tasks-automerge";

pub struct Document {
    pub frontend: Frontend,
    pub backend: Backend,
}

impl Document {
    pub fn new() -> Self {
        let mut backend = match LocalStorage::get(TASKS_STORAGE_KEY) {
            Ok(tasks) => Backend::load(tasks).unwrap(),
            Err(e) => {
                log!("err loading tasks", e);
                Backend::init()
            }
        };
        let mut frontend = Frontend::new();
        frontend.apply_patch(backend.get_patch().unwrap()).unwrap();
        if frontend.value_at_path(&Path::root().key("tasks")).is_none() {
            let changes = frontend
                .change::<_, automerge::InvalidChangeRequest>(None, |d| {
                    d.add_change(automerge::LocalChange::set(
                        Path::root().key("tasks"),
                        automerge::Value::Map(HashMap::new(), automerge_protocol::MapType::Map),
                    ))?;
                    Ok(())
                })
                .unwrap();
            if let Some(changes) = changes {
                let (patch, _) = backend.apply_local_change(changes).unwrap();
                frontend.apply_patch(patch).unwrap();
            }
        }
        Self { frontend, backend }
    }

    pub fn task(&self, uuid: &uuid::Uuid) -> Option<Task> {
        self.frontend
            .get_value(&Path::root().key("tasks").key(uuid.to_string()))
            .map(|v| Task::try_from(v).unwrap())
    }

    pub fn tasks(&self) -> HashMap<uuid::Uuid, Task> {
        self.frontend
            .get_value(&Path::root().key("tasks"))
            .map(|v| {
                if let automerge::Value::Map(map, automerge_protocol::MapType::Map) = v {
                    map.into_iter()
                        .map(|(k, v)| {
                            (
                                uuid::Uuid::from_str(&k).unwrap(),
                                Task::try_from(v).unwrap(),
                            )
                        })
                        .collect()
                } else {
                    panic!("expected a map")
                }
            })
            .unwrap_or_default()
    }

    pub fn change_task<F>(&mut self, uuid: &uuid::Uuid, f: F) -> Option<Msg>
    where
        F: FnOnce(Path, Task) -> Vec<automerge::LocalChange>,
    {
        let changes = self
            .frontend
            .change::<_, automerge::InvalidChangeRequest>(None, |d| {
                let task = d
                    .value_at_path(&Path::root().key("tasks").key(uuid.to_string()))
                    .map(|v| Task::try_from(v).unwrap());
                if let Some(task) = task {
                    let changes = f(Path::root().key("tasks").key(uuid.to_string()), task);
                    for change in changes {
                        d.add_change(change)?
                    }
                }
                Ok(())
            })
            .unwrap();
        changes.map(Msg::ApplyChange)
    }

    pub fn add_task(&mut self, uuid: uuid::Uuid) {
        let changes = self
            .frontend
            .change::<_, automerge::InvalidChangeRequest>(None, |d| {
                let task = d
                    .value_at_path(&Path::root().key("tasks").key(uuid.to_string()))
                    .map(|v| Task::try_from(v).unwrap());
                if task.is_none() {
                    let changes = Task::create(uuid);
                    for change in changes {
                        d.add_change(change).unwrap()
                    }
                }
                Ok(())
            })
            .unwrap();
        if let Some(changes) = changes {
            let (patch, _) = self.backend.apply_local_change(changes).unwrap();
            self.frontend.apply_patch(patch).unwrap();
        }
    }

    pub fn save(&self) -> Vec<u8> {
        self.backend.save().unwrap()
    }
}
