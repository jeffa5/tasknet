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
    selected_task: Option<Task>,
    new_task_description: String,
}

// ------ ------
//    Update
// ------ ------

enum Msg {
    SelectTask(Option<uuid::Uuid>),
    SelectedTaskDescriptionChanged(String),
    SaveSelectedTask,
    CreateTask,
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => model.selected_task = None,
        Msg::SelectTask(Some(uuid)) => {
            let task = model.tasks[&uuid].clone();
            model.selected_task = Some(task)
        }
        Msg::SelectedTaskDescriptionChanged(new_description) => {
            if let Some(task) = &mut model.selected_task {
                task.description = new_description;
            }
        }
        Msg::SaveSelectedTask => {
            if let Some(task) = model.selected_task.take() {
                model.tasks.insert(task.uuid, task);
            }
        }
        Msg::CreateTask => {
            let task = Task::new();
            model.selected_task = Some(task)
        }
    }
    LocalStorage::insert(STORAGE_KEY, &model.tasks).expect("save tasks to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    if let Some(ref task) = model.selected_task {
        div![
            C!["flex", "flex-col", "container", "mx-auto"],
            view_titlebar(model),
            view_selected_task(&task),
        ]
    } else {
        div![
            C!["flex", "flex-col", "container", "mx-auto"],
            view_titlebar(model),
            view_actions(model),
            view_tasks(&model.tasks),
        ]
    }
}

fn view_titlebar(_model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-between", "mb-4"],
        a![
            C!["bg-gray-50", "py-2", "px-8", "mr-8"],
            attrs! {At::Href => "/"},
            "logo"
        ],
        p![C!["bg-gray-50", "w-full", "py-2", "px-8", "mr-8"], "Search"],
        nav![C!["bg-gray-50", "py-2", "px-8"], "Nav", "Nav2"]
    ]
}

fn view_selected_task(task: &Task) -> Node<Msg> {
    div![
        C!["flex", "flex-col"],
        div![view_task_field(
            &task.description,
            Msg::SelectedTaskDescriptionChanged
        )],
        div![
            C!["flex", "justify-end"],
            button![
                C!["mr-4", "bg-gray-100", "py-2", "px-4"],
                mouse_ev(Ev::Click, |_| Msg::SelectTask(None)),
                "Cancel"
            ],
            button![
                C!["bg-gray-100", "py-2", "px-4"],
                mouse_ev(Ev::Click, |_| Msg::SaveSelectedTask),
                "Save"
            ]
        ]
    ]
}

fn view_task_field(value: &str, f: impl FnOnce(String) -> Msg + Clone + 'static) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "p-2", "mb-2"],
        div![C!["font-bold"], "Description"],
        input![
            C!["border"],
            attrs! {
                At::Value => value,
            },
            input_ev(Ev::Input, f)
        ]
    ]
}

fn view_actions(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-around"],
        view_filters(model),
        button![
            C!["bg-gray-100", "py-2", "px-4"],
            mouse_ev(Ev::Click, |_| Msg::CreateTask),
            "Create"
        ]
    ]
}

fn view_filters(_model: &Model) -> Node<Msg> {
    div![
        C!["bg-gray-50", "w-full", "py-2", "px-2", "mr-2"],
        "Filters"
    ]
}

fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>) -> Node<Msg> {
    let mut tasks: Vec<_> = tasks
        .iter()
        .map(|(_, t)| (urgency::calculate(t), t))
        .collect();
    // reverse sort so we have most urgent at the top
    tasks.sort_by(|(u1, _), (u2, _)| u2.partial_cmp(u1).unwrap());
    let rendered_tasks = tasks.iter().map(|(u, t)| view_task(t, *u));
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
            rendered_tasks
        ]
    ]
}

fn view_task(task: &Task, urgency: f64) -> Node<Msg> {
    let age = (chrono::offset::Utc::now()).signed_duration_since(task.entry);
    let id = task.uuid;
    tr![
        mouse_ev(Ev::Click, move |_event| {
            log!("clicked");
            Msg::SelectTask(Some(id))
        }),
        td![C!["text-center", "px-2"], view_duration(age)],
        td![C!["border-l-2", "text-left", "px-2"], &task.description],
        td![
            C!["border-l-2", "text-center", "px-2"],
            format!("{:.2}", urgency)
        ]
    ]
}

fn view_duration(duration: chrono::Duration) -> Node<Msg> {
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
