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
    pub fn change_task<F>(&mut self, uuid: &uuid::Uuid, f: F) -> Option<Msg>
    where
        F: FnOnce(&mut Task),
    {
        let changes = self
            .inner
            .change::<_, automerge::InvalidChangeRequest>(|d| {
                let task = d.tasks.get_mut(&Id(*uuid));
                if let Some(task) = task {
                    f(task)
                }
                Ok(())
            })
            .unwrap();
        changes.map(Msg::ApplyChange)
    }

    #[must_use]
    pub fn add_task(&mut self, uuid: uuid::Uuid) -> Option<Msg> {
        log!("getting changes");
        let change_result = self
            .inner
            .change::<_, automerge::InvalidChangeRequest>(|d| {
                let task = d.tasks.get(&Id(uuid));
                log!("task", task);
                if task.is_none() {
                    d.tasks.insert(Id(uuid), Task::new());
                    log!("inserted task");
                }
                Ok(())
            });
        log!("change result", change_result);
        let changes = change_result.unwrap();
        log!("got changes");
        changes.map(Msg::ApplyChange)
    }

    pub fn save(&self) -> Vec<u8> {
        self.backend.save().unwrap()
    }
}
