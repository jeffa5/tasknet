use std::collections::HashMap;

use automergeable::{automerge, Automergeable};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    task::{Id, Task},
    Msg,
};

pub const TASKS_STORAGE_KEY: &str = "tasknet-automerge";

pub struct Document {
    pub inner: automergeable::Document<DocumentInner>,
    pub backend: automerge::Backend,
}

#[derive(Debug, Default, Clone, Automergeable)]
pub struct DocumentInner {
    tasks: HashMap<Id, Task>,
}

impl Document {
    pub fn new() -> Self {
        let backend = match LocalStorage::get(TASKS_STORAGE_KEY) {
            Ok(tasks) => automerge::Backend::load(tasks).unwrap(),
            Err(e) => {
                log!("err loading tasks", e);
                automerge::Backend::init()
            }
        };
        let patch = backend.get_patch().unwrap();
        let mut inner =
            automergeable::Document::<DocumentInner>::new_with_timestamper(Box::new(|| {
                Some(chrono::Utc::now().timestamp())
            }));
        inner.apply_patch(patch).unwrap();
        Self { inner, backend }
    }

    pub fn task(&self, uuid: &uuid::Uuid) -> Option<Task> {
        self.inner
            .get()
            .and_then(|v| v.tasks.get(&Id(*uuid)).cloned())
    }

    pub fn tasks(&self) -> HashMap<Id, Task> {
        self.inner.get().map(|v| v.tasks).unwrap_or_default()
    }

    #[must_use]
    pub fn set_task(&mut self, uuid: uuid::Uuid, task: Task) -> Option<Msg> {
        let change_result = self
            .inner
            .change::<_, automerge::InvalidChangeRequest>(|d| {
                d.tasks.insert(Id(uuid), task);
                Ok(())
            });
        let change = change_result.unwrap();
        change.map(Msg::ApplyChange)
    }

    #[must_use]
    pub fn remove_task(&mut self, uuid: uuid::Uuid) -> Option<Msg> {
        let change_result = self
            .inner
            .change::<_, automerge::InvalidChangeRequest>(|d| {
                d.tasks.remove(&Id(uuid));
                Ok(())
            });
        let change = change_result.unwrap();
        change.map(Msg::ApplyChange)
    }

    pub fn save(&self) -> Vec<u8> {
        self.backend.save().unwrap()
    }
}
