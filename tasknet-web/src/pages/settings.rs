use std::collections::HashMap;

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{view_button, view_number_input_tr, view_text_input},
    task::Task,
    GlobalModel, Msg as GMsg, SETTINGS_STORAGE_KEY,
};

pub fn init(global_model: &GlobalModel) -> Model {
    Model {
        document_id: global_model.settings.document_id.to_string(),
    }
}

#[derive(Debug)]
pub struct Model {
    document_id: String,
}

#[derive(Debug, Clone)]
pub enum Msg {
    ImportTasks,
    ExportTasks,
    SetDocumentId(String),
    SetUrgencyNext(i64),
    SetUrgencyDue(i64),
    SetUrgencyHighPriority(i64),
    SetUrgencyMediumPriority(i64),
    SetUrgencyLowPriority(i64),
    SetUrgencyScheduled(i64),
    SetUrgencyActive(i64),
    SetUrgencyAge(i64),
    SetUrgencyNotes(i64),
    SetUrgencyTags(i64),
    SetUrgencyProject(i64),
    SetUrgencyWaiting(i64),
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
        Msg::SetDocumentId(new_id) => {
            model.document_id = new_id;

            match uuid::Uuid::parse_str(&model.document_id) {
                Ok(uuid) => {
                    global_model.settings.document_id = uuid;
                    orders.send_msg(GMsg::ChangeDocument);
                }
                Err(e) => {
                    log!("Error in document id:", e);
                }
            }
        }
        Msg::SetUrgencyNext(v) => global_model.settings.urgency.next = v as f64,
        Msg::SetUrgencyDue(v) => global_model.settings.urgency.due = v as f64,
        Msg::SetUrgencyHighPriority(v) => global_model.settings.urgency.high_priority = v as f64,
        Msg::SetUrgencyMediumPriority(v) => {
            global_model.settings.urgency.medium_priority = v as f64
        }
        Msg::SetUrgencyLowPriority(v) => global_model.settings.urgency.low_priority = v as f64,
        Msg::SetUrgencyScheduled(v) => global_model.settings.urgency.scheduled = v as f64,
        Msg::SetUrgencyActive(v) => global_model.settings.urgency.active = v as f64,
        Msg::SetUrgencyAge(v) => global_model.settings.urgency.age = v as f64,
        Msg::SetUrgencyNotes(v) => global_model.settings.urgency.notes = v as f64,
        Msg::SetUrgencyTags(v) => global_model.settings.urgency.tags = v as f64,
        Msg::SetUrgencyProject(v) => global_model.settings.urgency.project = v as f64,
        Msg::SetUrgencyWaiting(v) => global_model.settings.urgency.waiting = v as f64,
    }
    LocalStorage::insert(SETTINGS_STORAGE_KEY, &global_model.settings)
        .expect("save settings to LocalStorage");
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![
        C!["flex", "flex-col"],
        h1![C!["text-lg", "font-bold"], "Settings"],
        div![view_button(
            "Import Tasks",
            GMsg::Settings(Msg::ImportTasks),
            false
        ),],
        div![view_button(
            "Export Tasks",
            GMsg::Settings(Msg::ExportTasks),
            false
        ),],
        div![view_text_input(
            "Document ID",
            &model.document_id,
            &global_model.settings.document_id.to_string(),
            false,
            Default::default(),
            |s| GMsg::Settings(Msg::SetDocumentId(s))
        ),],
        view_urgency_coefficients(global_model, model),
    ]
}

fn view_urgency_coefficients(global_model: &GlobalModel, _model: &Model) -> Node<GMsg> {
    div![
        C!["flex", "flex-col"],
        h1![C!["text-lg", "font-bold"], "Urgency coefficients"],
        div![
            C!["ml-2"],
            table![
                C!["table-auto", "w-auto"],
                view_number_input_tr("Next", global_model.settings.urgency.next as i64, 15, |s| {
                    GMsg::Settings(Msg::SetUrgencyNext(s))
                }),
                view_number_input_tr("Due", global_model.settings.urgency.due as i64, 12, |s| {
                    GMsg::Settings(Msg::SetUrgencyDue(s))
                }),
                view_number_input_tr(
                    "High priority",
                    global_model.settings.urgency.high_priority as i64,
                    6,
                    |s| { GMsg::Settings(Msg::SetUrgencyHighPriority(s)) }
                ),
                view_number_input_tr(
                    "Medium priority",
                    global_model.settings.urgency.medium_priority as i64,
                    4,
                    |s| { GMsg::Settings(Msg::SetUrgencyMediumPriority(s)) }
                ),
                view_number_input_tr(
                    "Low priority",
                    global_model.settings.urgency.low_priority as i64,
                    2,
                    |s| { GMsg::Settings(Msg::SetUrgencyLowPriority(s)) }
                ),
                view_number_input_tr(
                    "Scheduled",
                    global_model.settings.urgency.scheduled as i64,
                    5,
                    |s| { GMsg::Settings(Msg::SetUrgencyScheduled(s)) }
                ),
                view_number_input_tr(
                    "Active",
                    global_model.settings.urgency.active as i64,
                    4,
                    |s| { GMsg::Settings(Msg::SetUrgencyActive(s)) }
                ),
                view_number_input_tr("Age", global_model.settings.urgency.age as i64, 2, |s| {
                    GMsg::Settings(Msg::SetUrgencyAge(s))
                }),
                view_number_input_tr(
                    "Notes",
                    global_model.settings.urgency.notes as i64,
                    1,
                    |s| { GMsg::Settings(Msg::SetUrgencyNotes(s)) }
                ),
                view_number_input_tr("Tags", global_model.settings.urgency.tags as i64, 1, |s| {
                    GMsg::Settings(Msg::SetUrgencyTags(s))
                }),
                view_number_input_tr(
                    "Project",
                    global_model.settings.urgency.project as i64,
                    1,
                    |s| { GMsg::Settings(Msg::SetUrgencyProject(s)) }
                ),
                view_number_input_tr(
                    "Waiting",
                    global_model.settings.urgency.waiting as i64,
                    -3,
                    |s| { GMsg::Settings(Msg::SetUrgencyWaiting(s)) }
                ),
            ],
        ],
    ]
}
