use seed::prelude::{LocalStorage, WebStorage};

use crate::task::Task;
use std::collections::HashMap;

const TASKS_STORAGE_KEY: &str = "tasknet-tasks";

#[derive(Debug)]
pub struct Document {
    pub tasks: HashMap<uuid::Uuid, Task>,
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
        Self { tasks }
    }

    pub fn save(&self) {
        LocalStorage::insert(TASKS_STORAGE_KEY, &self.tasks).expect("save tasks to LocalStorage");
    }
}
