use std::collections::HashMap;

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::view_button_str,
    task::{Task, TaskId},
    GlobalModel, Msg as GMsg,
};

pub fn init() -> Model {
    Model {}
}

#[derive(Debug)]
pub struct Model {}

#[derive(Clone)]
pub enum Msg {
    ImportTasks,
    ExportTasks,
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub fn update(
    msg: Msg,
    global_model: &mut GlobalModel,
    _model: &mut Model,
    _orders: &mut impl Orders<GMsg>,
) {
    match msg {
        Msg::ImportTasks => match window().prompt_with_message("Paste the tasks json here") {
            Ok(Some(content)) => match serde_json::from_str::<HashMap<TaskId, Task>>(&content) {
                Ok(tasks) => {
                    for task in tasks.into_values() {
                        global_model.document.update_task(task);
                    }
                }
                Err(e) => {
                    log!(e);
                    window()
                        .alert_with_message("Failed to import tasks")
                        .unwrap_or_else(|e| log!(e));
                }
            },
            Ok(None) => {}
            Err(e) => {
                log!(e);
                window()
                    .alert_with_message("Failed to create prompt")
                    .unwrap_or_else(|e| log!(e));
            }
        },
        Msg::ExportTasks => {
            let json = serde_json::to_string(&global_model.document.tasks());
            match json {
                Ok(json) => {
                    window()
                        .prompt_with_message_and_default("Copy this", &json)
                        .unwrap_or_else(|e| {
                            log!(e);
                            None
                        });
                }
                Err(e) => log!(e),
            }
        }
    }
}

pub fn view(_global_model: &GlobalModel, _model: &Model) -> Node<GMsg> {
    div![
        C![
            "flex",
            "flex-col",
            "mx-auto",
            "bg-gray-100",
            "p-2",
            "border-4",
            "border-gray-200",
        ],
        div![C!["mx-auto"], "Settings"],
        view_button_str("Import Tasks", GMsg::Settings(Msg::ImportTasks)),
        view_button_str("Export Tasks", GMsg::Settings(Msg::ExportTasks)),
    ]
}
