use automerge::AutoCommit;
use seed::prelude::{LocalStorage, WebStorage};

use crate::task::Task;
use std::collections::HashMap;

const TASKS_STORAGE_KEY: &str = "tasknet-tasks";
const AUTODOC_STORAGE_KEY: &str = "tasknet-autodoc";

#[derive(Debug)]
pub struct Document {
    pub tasks: HashMap<uuid::Uuid, Task>,
    pub autodoc: automerge::AutoCommit,
}

impl Document {
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
