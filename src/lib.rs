#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};
use std::collections::HashMap;

mod task;
mod urgency;

use task::Task;

// ------ ------
//     Init
// ------ ------

const STORAGE_KEY: &str = "tasknet-tasks";

fn init(_url: Url, _orders: &mut impl Orders<Msg>) -> Model {
    Model {
        tasks: LocalStorage::get(STORAGE_KEY).unwrap_or_default(),
        selected_task: None,
        new_task_description: String::new(),
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
struct Model {
    tasks: HashMap<uuid::Uuid, Task>,
    selected_task: Option<uuid::Uuid>,
    new_task_description: String,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    SelectTask(Option<uuid::Uuid>),
    SelectedTaskDescriptionChanged(String),
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            log!("no selected task")
        }
        Msg::SelectTask(Some(uuid)) => {
            log!("selecting", uuid)
        }
        Msg::SelectedTaskDescriptionChanged(new_description) => {
            log!(new_description);
            match model.selected_task {
                None => unreachable!(),
                Some(ref uuid) => {
                    let task = model.tasks.get_mut(uuid).unwrap();
                    task.description = new_description;
                }
            }
        }
    }
    LocalStorage::insert(STORAGE_KEY, &model.tasks).expect("save tasks to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "container", "mx-auto"],
        view_titlebar(model),
        view_filters(model),
        view_tasks(&model.tasks),
    ]
}

fn view_titlebar(_model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-around", "mb-4"],
        div![C!["bg-gray-50", "px-8"], "logo"],
        p![C!["bg-gray-50", "px-8"], "Search"]
    ]
}

fn view_filters(_model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-around", "bg-gray-50"],
        "Filters"
    ]
}

fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>) -> Node<Msg> {
    let tasks = tasks.iter().map(|(_, t)| view_task(t));
    div![
        C!["mt-8"],
        table![
            C!["table-auto", "w-full"],
            tr![
                C!["border-b-2"],
                th!["Age"],
                th![C!["border-l-2"], "Description"],
                th![C!["border-l-2"], "Urgency"]
            ],
            tasks
        ]
    ]
}

fn view_task(task: &Task) -> Node<Msg> {
    let age = (chrono::offset::Utc::now()).signed_duration_since(task.entry);
    let urgency = urgency::calculate(task);
    let id = task.uuid;
    tr![
        mouse_ev(Ev::Click, move |_event| {
            log!("clicked");
            Msg::SelectTask(Some(id))
        }),
        td![C!["text-center", "px-2"], view_duration(age)],
        td![C!["border-l-2", "text-left", "px-2"], &task.description],
        td![C!["border-l-2", "text-center", "px-2"], urgency]
    ]
}

fn view_duration(duration: chrono::Duration) -> Node<Msg> {
    log!(duration);
    let s = if duration.num_weeks() > 0 {
        format!("{}w", duration.num_weeks())
    } else if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_seconds() > 0 {
        format!("{}s", duration.num_seconds())
    } else {
        "now".to_owned()
    };
    plain!(s)
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
