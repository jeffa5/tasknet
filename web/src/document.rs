use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use seed::{
    log,
    prelude::{LocalStorage, WebStorage},
};

use crate::task::{Task, TaskId};
use std::collections::HashMap;

const AUTODOC_STORAGE_KEY: &str = "tasknet-autodoc";

#[derive(Debug)]
pub struct Document {
    tasks: HashMap<TaskId, Task>,
    autodoc: automerge::AutoCommit,
    server_sync_state: automerge::sync::State,
}

impl Document {
    pub fn get_task(&self, id: &TaskId) -> Option<&Task> {
        self.tasks.get(id)
    }

    pub const fn tasks(&self) -> &HashMap<TaskId, Task> {
        &self.tasks
    }

    pub fn new_task(&mut self) -> TaskId {
        let task = Task::new();
        let id = task.id().clone();
        self.tasks.insert(id.clone(), task);
        reconcile(&mut self.autodoc, &self.tasks).unwrap();
        id
    }

    pub fn change_task<F: FnOnce(&mut Task)>(&mut self, id: &TaskId, f: F) {
        if let Some(task) = self.tasks.get_mut(id) {
            f(task);
            reconcile(&mut self.autodoc, &self.tasks).unwrap();
        }
    }

    pub fn update_task(&mut self, task: Task) {
        self.tasks.insert(task.id().clone(), task);
        reconcile(&mut self.autodoc, &self.tasks).unwrap();
    }

    pub fn remove_task(&mut self, id: &TaskId) {
        self.tasks.remove(id);
        reconcile(&mut self.autodoc, &self.tasks).unwrap();
    }

    pub fn load() -> Self {
        let saved_document: String =
            LocalStorage::get(AUTODOC_STORAGE_KEY).map_or_else(|_| String::new(), |bytes| bytes);
        let saved_document = base64::decode(saved_document).unwrap_or_default();
        let autodoc = AutoCommit::load(&saved_document).unwrap_or_else(|_| AutoCommit::new());
        let tasks = hydrate(&autodoc).unwrap();
        Self {
            tasks,
            autodoc,
            server_sync_state: automerge::sync::State::default(),
        }
    }

    pub fn save(&mut self) {
        let bytes = self.autodoc.save();
        let bytes = base64::encode(bytes);
        LocalStorage::insert(AUTODOC_STORAGE_KEY, &bytes).expect("save autodoc to LocalStorage");
    }

    pub fn generate_sync_message(&mut self) -> Option<Vec<u8>> {
        self.autodoc
            .generate_sync_message(&mut self.server_sync_state)
            .map(automerge::sync::Message::encode)
    }

    pub fn receive_sync_message(&mut self, message: &[u8]) {
        match automerge::sync::Message::decode(message) {
            Ok(message) => {
                if let Err(err) = self
                    .autodoc
                    .receive_sync_message(&mut self.server_sync_state, message)
                {
                    log!("Failed to receive sync message from server: ", err);
                }
            }
            Err(err) => {
                log!("Failed to decode sync message:", err);
            }
        }
    }
}
