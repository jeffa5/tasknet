use std::collections::HashMap;

use automergeable::{automerge, Automergeable};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    task::{Id, Task},
    Msg,
};

const TASKS_STORAGE_KEY: &str = "tasknet-tasks";

pub struct Document {
    pub inner: automergeable::Document<DocumentInner>,
    pub backend: automerge_persistent::PersistentBackend<
        automerge_persistent_localstorage::LocalStoragePersister,
    >,
}

#[derive(Debug, Default, Clone, Automergeable)]
pub struct DocumentInner {
    tasks: HashMap<Id, Task>,
}

impl Document {
    pub fn new() -> Self {
        let persister = automerge_persistent_localstorage::LocalStoragePersister::new(
            LocalStorage::storage().unwrap(),
            "automerge-persistent-document".to_owned(),
            "automerge-persistent-changes".to_owned(),
        );
        log!("loading");
        let backend = automerge_persistent::PersistentBackend::load(persister).unwrap();
        log!("loaded");
        let patch = backend.get_patch().unwrap();
        log!("got patch");
        let mut inner =
            automergeable::Document::<DocumentInner>::new_with_timestamper(Box::new(|| {
                Some(chrono::Utc::now().timestamp())
            }));
        log!("made document");
        inner.apply_patch(patch).unwrap();
        log!("applied patch");
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
        LocalStorage::insert(TASKS_STORAGE_KEY, &self.tasks()).expect("save tasks to LocalStorage");
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
        LocalStorage::insert(TASKS_STORAGE_KEY, &self.tasks()).expect("save tasks to LocalStorage");
        change.map(Msg::ApplyChange)
    }
}
