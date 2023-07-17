use automerge::sync::SyncDoc;
use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use base64::Engine;
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
        let b64_engine = base64::engine::general_purpose::STANDARD;
        let saved_document = b64_engine.decode(saved_document).unwrap_or_default();
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
        let b64_engine = base64::engine::general_purpose::STANDARD;
        let bytes = b64_engine.encode(bytes);
        LocalStorage::insert(AUTODOC_STORAGE_KEY, &bytes).expect("save autodoc to LocalStorage");
    }

    pub fn generate_sync_message(&mut self) -> Option<Vec<u8>> {
        self.autodoc
            .sync()
            .generate_sync_message(&mut self.server_sync_state)
            .map(automerge::sync::Message::encode)
    }

    pub fn receive_sync_message(&mut self, message: &[u8]) {
        match automerge::sync::Message::decode(message) {
            Ok(message) => {
                let res = self
                    .autodoc
                    .sync()
                    .receive_sync_message(&mut self.server_sync_state, message);
                match res {
                    Ok(()) => {
                        self.tasks = hydrate(&self.autodoc).unwrap();
                    }
                    Err(err) => {
                        log!("Failed to receive sync message from server: ", err);
                    }
                }
            }
            Err(err) => {
                log!("Failed to decode sync message:", err);
            }
        }
    }
}
