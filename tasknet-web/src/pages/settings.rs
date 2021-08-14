use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_checkbox, view_text_input},
    settings::Settings,
    task::{Id, Priority, Status, Task},
    urgency, Filters, GlobalModel, Msg as GMsg,
};

const SETTINGS_STORAGE_KEY: &str = "tasknet-filters";

pub fn init() -> Model {
    let settings = match LocalStorage::get(SETTINGS_STORAGE_KEY) {
        Ok(settings) => settings,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse settings: {:?}", err)
        }
        Err(_) => Settings::default(),
    };
    Model { settings }
}

#[derive(Debug)]
pub struct Model {
    settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Msg {
    ImportTasks,
    ExportTasks,
}

#[allow(clippy::too_many_lines)]
pub fn update(
    msg: Msg,
    global_model: &mut GlobalModel,
    model: &mut Model,
    orders: &mut impl Orders<GMsg>,
) {
    match msg {
        Msg::ImportTasks => {
            let tasks: HashMap<uuid::Uuid, Task> = serde_json::from_str(
                &window()
                    .prompt_with_message("Paste the tasks json here")
                    .unwrap()
                    .unwrap_or_else(|| "{}".to_owned()),
            )
            .unwrap();
            log!("importing", tasks.len(), "tasks");
            let msg = global_model.document.set_tasks(tasks);
            if let Some(msg) = msg {
                orders.skip().send_msg(msg);
            }
        }
        Msg::ExportTasks => {
            let tasks = global_model.document.tasks();
            window()
                .prompt_with_message_and_default(
                    "Copy this",
                    &serde_json::to_string(&tasks).unwrap(),
                )
                .unwrap();
        }
    }
    LocalStorage::insert(SETTINGS_STORAGE_KEY, &model.settings)
        .expect("save settings to LocalStorage");
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![
        C!["flex", "flex-col"],
        h1!["Settings"],
        view_button("Import Tasks", GMsg::Settings(Msg::ImportTasks), false),
        view_button("Export Tasks", GMsg::Settings(Msg::ExportTasks), false),
    ]
}
