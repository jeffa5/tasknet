#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};
use std::collections::HashMap;
use std::collections::HashSet;

mod task;
mod urgency;

use task::Task;

// ------ ------
//     Init
// ------ ------

const STORAGE_KEY: &str = "tasknet-tasks";

fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::interval(1000, || Msg::OnTick));
    Model {
        tasks: LocalStorage::get(STORAGE_KEY).unwrap(),
        selected_task: None,
        new_task_description: String::new(),
        filters: Filters::default(),
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
    filters: Filters,
}

#[derive(Debug)]
struct Filters {
    status_pending: bool,
    status_completed: bool,
    status_deleted: bool,
    status_waiting: bool,
    project: Vec<String>,
    tags: Vec<String>,
    description: String,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            status_pending: true,
            status_completed: false,
            status_deleted: false,
            status_waiting: false,
            project: Vec::new(),
            tags: Vec::new(),
            description: String::new(),
        }
    }
}

impl Filters {
    fn filter_task(&self, task: &Task) -> bool {
        let filter_status = match task {
            Task::Pending(_) => self.status_pending,
            Task::Deleted(_) => self.status_deleted,
            Task::Completed(_) => self.status_completed,
            Task::Waiting(_) => self.status_waiting,
        };
        let filter_project = task
            .project()
            .join(".")
            .starts_with(&self.project.join("."));
        let filter_tags = self
            .tags
            .iter()
            .all(|tag| task.tags().iter().any(|t| t.starts_with(tag)));
        let filter_description = task.description().contains(&self.description);
        filter_status && filter_project && filter_tags && filter_description
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
enum Msg {
    SelectTask(Option<uuid::Uuid>),
    SelectedTaskDescriptionChanged(String),
    SelectedTaskProjectChanged(String),
    SelectedTaskTagsChanged(String),
    CreateTask,
    DeleteSelectedTask,
    CompleteSelectedTask,
    StartSelectedTask,
    StopSelectedTask,
    OnTick,
    FiltersStatusTogglePending,
    FiltersStatusToggleDeleted,
    FiltersStatusToggleCompleted,
    FiltersStatusToggleWaiting,
    FiltersProjectChanged(String),
    FiltersTagsChanged(String),
    FiltersDescriptionChanged(String),
    FiltersReset,
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => model.selected_task = None,
        Msg::SelectTask(Some(uuid)) => model.selected_task = Some(uuid),
        Msg::SelectedTaskDescriptionChanged(new_description) => {
            if let Some(uuid) = model.selected_task {
                if let Some(task) = &mut model.tasks.get_mut(&uuid) {
                    task.set_description(new_description);
                }
            }
        }
        Msg::SelectedTaskProjectChanged(new_project) => {
            let new_project = new_project.trim();
            if let Some(uuid) = model.selected_task {
                if let Some(task) = &mut model.tasks.get_mut(&uuid) {
                    task.set_project(if new_project.is_empty() {
                        Vec::new()
                    } else {
                        new_project.split('.').map(|s| s.to_owned()).collect()
                    })
                }
            }
        }
        Msg::SelectedTaskTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            if let Some(uuid) = model.selected_task {
                if let Some(task) = &mut model.tasks.get_mut(&uuid) {
                    task.set_tags(if new_tags.is_empty() {
                        Vec::new()
                    } else {
                        let mut tags: Vec<_> = new_tags
                            .split_whitespace()
                            .map(|s| s.trim().to_owned())
                            .collect();
                        if new_end {
                            tags.push(String::new())
                        }
                        tags
                    })
                }
            }
        }
        Msg::CreateTask => {
            let task = Task::new();
            let id = task.uuid();
            model.tasks.insert(task.uuid(), task);
            model.selected_task = Some(id)
        }
        Msg::DeleteSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                if let Some(task) = model.tasks.remove(&uuid) {
                    match task {
                        Task::Pending(_) | Task::Completed(_) | Task::Waiting(_) => {
                            model.tasks.insert(task.uuid(), task.delete());
                        }
                        Task::Deleted(task) => {
                            model.tasks.remove(&task.uuid());
                        }
                    }
                }
            }
        }
        Msg::CompleteSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                if let Some(task) = model.tasks.remove(&uuid) {
                    model.tasks.insert(task.uuid(), task.complete());
                }
            }
        }
        Msg::StartSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                if let Some(task) = model.tasks.get_mut(&uuid) {
                    match task {
                        Task::Pending(task) => {
                            task.activate();
                        }
                        Task::Deleted(_) => {}
                        Task::Completed(_) => {}
                        Task::Waiting(_) => {}
                    };
                }
            }
        }
        Msg::StopSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                if let Some(task) = model.tasks.get_mut(&uuid) {
                    match task {
                        Task::Pending(task) => {
                            task.deactivate();
                        }
                        Task::Deleted(_) => {}
                        Task::Completed(_) => {}
                        Task::Waiting(_) => {}
                    };
                }
            }
        }
        Msg::OnTick => {
            // just re-render to show update ages
        }
        Msg::FiltersStatusTogglePending => {
            model.filters.status_pending = !model.filters.status_pending
        }
        Msg::FiltersStatusToggleDeleted => {
            model.filters.status_deleted = !model.filters.status_deleted
        }
        Msg::FiltersStatusToggleCompleted => {
            model.filters.status_completed = !model.filters.status_completed
        }
        Msg::FiltersStatusToggleWaiting => {
            model.filters.status_waiting = !model.filters.status_waiting
        }
        Msg::FiltersProjectChanged(new_project) => {
            let new_project = new_project.trim();
            model.filters.project = if new_project.is_empty() {
                Vec::new()
            } else {
                new_project.split('.').map(|s| s.to_owned()).collect()
            }
        }
        Msg::FiltersTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            model.filters.tags = if new_tags.is_empty() {
                Vec::new()
            } else {
                let mut tags: Vec<_> = new_tags
                    .split_whitespace()
                    .map(|s| s.trim().to_owned())
                    .collect();
                if new_end {
                    tags.push(String::new())
                }
                tags
            }
        }
        Msg::FiltersDescriptionChanged(new_description) => {
            model.filters.description = new_description
        }
        Msg::FiltersReset => model.filters = Filters::default(),
    }
    LocalStorage::insert(STORAGE_KEY, &model.tasks).expect("save tasks to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    if let Some(uuid) = model.selected_task {
        if let Some(task) = model.tasks.get(&uuid) {
            div![
                C!["flex", "flex-col", "container", "mx-auto"],
                view_titlebar(),
                view_selected_task(task),
            ]
        } else {
            empty![]
        }
    } else {
        div![
            C!["flex", "flex-col", "container", "mx-auto"],
            view_titlebar(),
            view_actions(model),
            view_tasks(&model.tasks, &model.filters),
        ]
    }
}

fn view_titlebar() -> Node<Msg> {
    div![
        C!["flex", "justify-center", "mb-4"],
        a![
            C!["bg-gray-200", "py-2", "px-8", "mr-8", "hover:bg-gray-300",],
            attrs! {At::Href => "/"},
            "TaskNet"
        ],
    ]
}

fn view_selected_task(task: &Task) -> Node<Msg> {
    let is_pending = matches!(task, Task::Pending(_));
    let start = match task {
        Task::Pending(task) => *task.start(),
        Task::Deleted(_) | Task::Completed(_) | Task::Waiting(_) => None,
    };
    let end = match task {
        Task::Completed(task) => Some(task.end()),
        Task::Deleted(task) => Some(task.end()),
        Task::Pending(_) | Task::Waiting(_) => None,
    };
    let urgency = urgency::calculate(task);
    div![
        C!["flex", "flex-col"],
        div![
            C!["pl-2"],
            span![C!["font-bold"], "Status: "],
            match task {
                Task::Pending(_) => "Pending",
                Task::Deleted(_) => "Deleted",
                Task::Completed(_) => "Completed",
                Task::Waiting(_) => "Waiting",
            }
        ],
        IF!(urgency.is_some() => div![
            C!["pl-2"],
            span![C!["font-bold"], "Urgency: "],
            if let Some(urgency) = urgency {
                plain![format!("{:.2}", urgency)]
            } else {
                empty![]
            }
        ]),
        div![
            C!["pl-2"],
            span![C!["font-bold"], "Entry: "],
            task.entry().to_string()
        ],
        if let Some(start) = start {
            div![
                C!["pl-2"],
                span![C!["font-bold"], "Start: "],
                start.to_string()
            ]
        } else {
            empty![]
        },
        if let Some(end) = end {
            div![C!["pl-2"], span![C!["font-bold"], "End: "], end.to_string()]
        } else {
            empty![]
        },
        div![view_text_input(
            "Description",
            &task.description(),
            Msg::SelectedTaskDescriptionChanged
        )],
        div![view_text_input(
            "Project",
            &task.project().join("."),
            Msg::SelectedTaskProjectChanged
        )],
        div![view_text_input(
            "Tags",
            &task.tags().join(" "),
            Msg::SelectedTaskTagsChanged
        )],
        div![
            C!["flex", "justify-end"],
            IF!(is_pending =>
                div![
                    C!["mr-4"],
                    if start.is_some() {
                        view_button("Stop", Msg::StopSelectedTask)
                    } else {
                        view_button("Start", Msg::StartSelectedTask)
                    }
                ]
            ),
            IF!(is_pending =>
                div![C!["mr-4"], view_button("Complete", Msg::CompleteSelectedTask)]
            ),
            IF!(!matches!(task, Task::Completed(_)) =>
                div![C!["mr-4"], view_button("Delete", Msg::DeleteSelectedTask)]
            ),
            view_button("Close", Msg::SelectTask(None))
        ]
    ]
}

fn view_text_input(
    name: &str,
    value: &str,
    f: impl FnOnce(String) -> Msg + Clone + 'static,
) -> Node<Msg> {
    let also_f = f.clone();
    div![
        C!["flex", "flex-col", "px-2", "mb-2"],
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

fn view_button(text: &str, msg: Msg) -> Node<Msg> {
    button![
        C!["bg-gray-200", "py-2", "px-4", "hover:bg-gray-300"],
        mouse_ev(Ev::Click, |_| msg),
        text
    ]
}

fn view_actions(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-around"],
        view_filters(&model.filters),
        view_button("Create", Msg::CreateTask)
    ]
}

fn view_filters(filters: &Filters) -> Node<Msg> {
    div![
        C![
            "flex",
            "flex-row",
            "bg-gray-50",
            "w-full",
            "py-2",
            "px-2",
            "mr-2"
        ],
        div![
            C!["flex", "flex-col", "mr-8"],
            h2![C!["font-bold"], "Status"],
            view_checkbox(
                "filters-status-pending",
                "Pending",
                filters.status_pending,
                Msg::FiltersStatusTogglePending
            ),
            view_checkbox(
                "filters-status-deleted",
                "Deleted",
                filters.status_deleted,
                Msg::FiltersStatusToggleDeleted
            ),
            view_checkbox(
                "filters-status-completed",
                "Completed",
                filters.status_completed,
                Msg::FiltersStatusToggleCompleted
            ),
            view_checkbox(
                "filters-status-waiting",
                "Waiting",
                filters.status_waiting,
                Msg::FiltersStatusToggleWaiting
            ),
        ],
        view_text_input(
            "Description",
            &filters.description,
            Msg::FiltersDescriptionChanged
        ),
        view_text_input(
            "Project",
            &filters.project.join("."),
            Msg::FiltersProjectChanged
        ),
        view_text_input("Tags", &filters.tags.join(" "), Msg::FiltersTagsChanged),
        view_button("Reset Filters", Msg::FiltersReset),
    ]
}

fn view_checkbox(name: &str, title: &str, checked: bool, msg: Msg) -> Node<Msg> {
    let msg_clone = msg.clone();
    div![
        C!["flex", "flex-row"],
        input![
            C!["mr-2"],
            attrs! {
                At::Type => "checkbox",
                At::Name => name,
                At::Checked => if checked { AtValue::None } else { AtValue::Ignored },
            },
            mouse_ev(Ev::Click, |_| msg),
        ],
        label![
            attrs! {
                At::For => name,
            },
            mouse_ev(Ev::Click, |_| msg_clone),
            title
        ]
    ]
}

fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>, filters: &Filters) -> Node<Msg> {
    let mut tasks: Vec<_> = tasks
        .iter()
        .map(|(_, t)| t)
        .filter(|t| filters.filter_task(t))
        .map(|t| ViewableTask {
            age: duration_string((chrono::offset::Utc::now()).signed_duration_since(*t.entry())),
            status: match t {
                Task::Pending(_) => "Pending".to_owned(),
                Task::Completed(_) => "Completed".to_owned(),
                Task::Deleted(_) => "Deleted".to_owned(),
                Task::Waiting(_) => "Waiting".to_owned(),
            },
            project: t.project().to_owned(),
            description: t.description().to_owned(),
            urgency: urgency::calculate(t),
            uuid: t.uuid(),
            tags: t.tags().to_owned(),
            active: match t {
                Task::Pending(t) => t.start().is_some(),
                Task::Completed(_) | Task::Deleted(_) | Task::Waiting(_) => false,
            },
        })
        .collect();

    // reverse sort so we have most urgent at the top
    tasks.sort_by(|t1, t2| t2.urgency.partial_cmp(&t1.urgency).unwrap());
    let show_status = tasks
        .iter()
        .map(|t| &t.status)
        .collect::<HashSet<_>>()
        .len()
        > 1;
    let show_project = tasks.iter().any(|t| !t.project.is_empty());
    let show_tags = tasks.iter().any(|t| !t.tags.is_empty());
    div![
        C!["mt-8"],
        table![
            C!["table-auto", "w-full"],
            tr![
                C!["border-b-2"],
                th!["Age"],
                IF!(show_status => th![C!["border-l-2"], "Status"]),
                IF!(show_project => th![C!["border-l-2"], "Project"]),
                IF!(show_tags => th![C!["border-l-2"], "Tags"]),
                th![C!["border-l-2"], "Description"],
                th![C!["border-l-2"], "Urgency"]
            ],
            tasks.into_iter().enumerate().map(|(i, t)| {
                let id = t.uuid;
                let is_next = t.tags.contains(&"next".to_owned());
                tr![
                    C![
                        IF!(i % 2 == 0 => "bg-gray-100"),
                        "hover:bg-gray-200",
                        "cursor-pointer",
                        "select-none",
                        IF!(t.active => "bg-green-200"),
                        IF!(t.active => "hover:bg-green-400"),
                        IF!(is_next => "bg-blue-200"),
                        IF!(is_next => "hover:bg-blue-400"),
                    ],
                    mouse_ev(Ev::Click, move |_| { Msg::SelectTask(Some(id)) }),
                    td![C!["text-center", "px-2"], t.age.clone()],
                    IF!(show_status => td![C!["border-l-2","text-center", "px-2"], t.status]),
                    IF!(show_project => td![
                        C!["border-l-2", "text-left", "px-2"],
                        if t.project.is_empty(){
                            empty![]
                        } else {
                            plain!(t.project.join("."))
                        }
                    ]),
                    IF!(show_tags => td![
                        C!["border-l-2", "text-left", "px-2"],
                        if t.tags.is_empty(){
                            empty![]
                        } else {
                            plain!(t.tags.join(" "))
                        }
                    ]),
                    td![C!["border-l-2", "text-left", "px-2"], &t.description],
                    td![
                        C!["border-l-2", "text-center", "px-2"],
                        if let Some(urgency) = t.urgency {
                            plain![format!("{:.2}", urgency)]
                        } else {
                            empty![]
                        }
                    ]
                ]
            })
        ]
    ]
}

struct ViewableTask {
    uuid: uuid::Uuid,
    active: bool,
    status: String,
    age: String,
    project: Vec<String>,
    description: String,
    tags: Vec<String>,
    urgency: Option<f64>,
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
