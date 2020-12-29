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
    SelectedTaskProjectChanged(String),
    SaveSelectedTask,
    CreateTask,
    DeleteSelectedTask,
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
                task.set_description(new_description);
            }
        }
        Msg::SelectedTaskProjectChanged(new_project) => {
            let new_project = new_project.trim();
            if let Some(task) = &mut model.selected_task {
                task.set_project(if new_project.is_empty() {
                    Vec::new()
                } else {
                    new_project.split('.').map(|s| s.to_owned()).collect()
                })
            }
        }
        Msg::SaveSelectedTask => {
            if let Some(task) = model.selected_task.take() {
                model.tasks.insert(task.uuid(), task);
            }
        }
        Msg::CreateTask => {
            let task = Task::new();
            model.selected_task = Some(task)
        }
        Msg::DeleteSelectedTask => {
            if let Some(task) = model.selected_task.take() {
                model.tasks.remove(&task.uuid());
            }
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
            view_titlebar(),
            view_selected_task(&task),
        ]
    } else {
        div![
            C!["flex", "flex-col", "container", "mx-auto"],
            view_titlebar(),
            view_actions(),
            view_tasks(&model.tasks),
        ]
    }
}

fn view_titlebar() -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-between", "mb-4"],
        a![
            C!["bg-gray-50", "py-2", "px-8", "mr-8", "hover:bg-gray-300"],
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
            "Description",
            &task.description(),
            Msg::SelectedTaskDescriptionChanged
        )],
        div![view_task_field(
            "Project",
            &task.project().join("."),
            Msg::SelectedTaskProjectChanged
        )],
        div![
            C!["flex", "justify-end"],
            button![
                C!["mr-4", "bg-gray-100", "py-2", "px-4", "hover:bg-gray-300"],
                mouse_ev(Ev::Click, |_| Msg::DeleteSelectedTask),
                "Delete"
            ],
            button![
                C!["mr-4", "bg-gray-100", "py-2", "px-4", "hover:bg-gray-300"],
                mouse_ev(Ev::Click, |_| Msg::SelectTask(None)),
                "Cancel"
            ],
            button![
                C!["bg-gray-100", "py-2", "px-4", "hover:bg-gray-300"],
                mouse_ev(Ev::Click, |_| Msg::SaveSelectedTask),
                "Save"
            ]
        ]
    ]
}

fn view_task_field(
    name: &str,
    value: &str,
    f: impl FnOnce(String) -> Msg + Clone + 'static,
) -> Node<Msg> {
    let also_f = f.clone();
    div![
        C!["flex", "flex-col", "p-2", "mb-2"],
        div![C!["font-bold"], name],
        div![
            input![
                C!["border", "mr-2"],
                attrs! {
                    At::Value => value,
                },
                input_ev(Ev::Input, f)
            ],
            IF!(!value.is_empty() => button![
                mouse_ev(Ev::Click, |_| also_f(String::new())),
                div![C!["text-red-600"], "X"]
            ])
        ]
    ]
}

fn view_actions() -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-around"],
        view_filters(),
        button![
            C!["bg-gray-100", "py-2", "px-4", "hover:bg-gray-300"],
            mouse_ev(Ev::Click, |_| Msg::CreateTask),
            "Create"
        ]
    ]
}

fn view_filters() -> Node<Msg> {
    div![
        C!["bg-gray-50", "w-full", "py-2", "px-2", "mr-2"],
        "Filters"
    ]
}

fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>) -> Node<Msg> {
    let mut tasks: Vec<_> = tasks
        .iter()
        .map(|(_, t)| ViewableTask {
            age: duration_string((chrono::offset::Utc::now()).signed_duration_since(*t.entry())),
            project: t.project().to_owned(),
            description: t.description().to_owned(),
            urgency: urgency::calculate(t),
            uuid: t.uuid(),
        })
        .collect();

    // reverse sort so we have most urgent at the top
    tasks.sort_by(|t1, t2| t2.urgency.partial_cmp(&t1.urgency).unwrap());
    let show_project = tasks.iter().any(|t| !t.project.is_empty());
    div![
        C!["mt-8"],
        table![
            C!["table-auto", "w-full"],
            tr![
                C!["border-b-2"],
                th!["Age"],
                IF!(show_project => th![C!["border-l-2"], "Project"]),
                th![C!["border-l-2"], "Description"],
                th![C!["border-l-2"], "Urgency"]
            ],
            tasks.into_iter().map(|t| {
                let id = t.uuid;
                tr![
                    C!["hover:bg-gray-200", "cursor-pointer"],
                    mouse_ev(Ev::Click, move |_| { Msg::SelectTask(Some(id)) }),
                    td![C!["text-center", "px-2"], t.age.clone()],
                    IF!(show_project => td![
                        C!["border-l-2", "text-center", "px-2"],
                        if t.project.is_empty(){
                            empty![]
                        } else {
                            plain!(t.project.join("."))
                        }
                    ]),
                    td![C!["border-l-2", "text-left", "px-2"], &t.description],
                    td![
                        C!["border-l-2", "text-center", "px-2"],
                        format!("{:.2}", t.urgency)
                    ]
                ]
            })
        ]
    ]
}

struct ViewableTask {
    uuid: uuid::Uuid,
    age: String,
    project: Vec<String>,
    description: String,
    urgency: f64,
}

fn duration_string(duration: chrono::Duration) -> String {
    if duration.num_weeks() > 0 {
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
    }
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
