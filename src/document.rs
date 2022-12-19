use automerge::AutoCommit;
use autosurgeon::reconcile;
use seed::prelude::{LocalStorage, WebStorage};

use crate::task::{Task, TaskId};
use std::collections::HashMap;

const TASKS_STORAGE_KEY: &str = "tasknet-tasks";
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
        let tasks = match LocalStorage::get(TASKS_STORAGE_KEY) {
            Ok(tasks) => tasks,
            Err(seed::browser::web_storage::WebStorageError::JsonError(err)) => {
                panic!("failed to parse tasks: {:?}", err)
            }
            Err(_) => HashMap::new(),
        };
        let saved_document: Vec<u8> =
            LocalStorage::get(AUTODOC_STORAGE_KEY).map_or_else(|_| Vec::new(), |bytes| bytes);
        let autodoc = AutoCommit::load(&saved_document).unwrap_or_else(|_| AutoCommit::new());
        Self { tasks, autodoc }
    }

    pub fn save(&mut self) {
        LocalStorage::insert(TASKS_STORAGE_KEY, &self.tasks).expect("save tasks to LocalStorage");
        LocalStorage::insert(AUTODOC_STORAGE_KEY, &self.autodoc.save())
            .expect("save autodoc to LocalStorage");
    }
}
