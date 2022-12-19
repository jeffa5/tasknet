use automerge::AutoCommit;
use autosurgeon::{hydrate, reconcile};
use seed::prelude::{LocalStorage, WebStorage};

use crate::task::{Task, TaskId};
use std::collections::HashMap;

const AUTODOC_STORAGE_KEY: &str = "tasknet-autodoc";

#[derive(Debug)]
pub struct Document {
    tasks: HashMap<TaskId, Task>,
    autodoc: automerge::AutoCommit,
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
        let saved_document: String = LocalStorage::get(AUTODOC_STORAGE_KEY)
            .map_or_else(|_| Default::default(), |bytes| bytes);
        let saved_document = base64::decode(saved_document).unwrap_or_default();
        let autodoc = AutoCommit::load(&saved_document).unwrap_or_else(|_| AutoCommit::new());
        let tasks = hydrate(&autodoc).unwrap();
        Self { tasks, autodoc }
    }

    pub fn save(&mut self) {
        let bytes = self.autodoc.save();
        let bytes = base64::encode(bytes);
        LocalStorage::insert(AUTODOC_STORAGE_KEY, &bytes).expect("save autodoc to LocalStorage");
    }
}
