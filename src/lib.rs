#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use apply::Apply;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod filters;
mod task;
mod urgency;

use filters::Filters;
use task::{Priority, Task};

const ESCAPE_KEY: &str = "Escape";

const VIEW_TASK_SEARCH_KEY: &str = "viewtask";
const TASKS_STORAGE_KEY: &str = "tasknet-tasks";
const FILTERS_STORAGE_KEY: &str = "tasknet-filters";

// ------ ------
//     Init
// ------ ------

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let url_clone = url.clone();
    orders.perform_cmd(async move {
        let res = window()
            .navigator()
            .service_worker()
            .register(&format!("{}/{}", url_clone.path().join("/"), "service-worker.js"))
            .apply(JsFuture::from)
            .await;
        if let Err(e) = res {
            log!("Error registering service worker:", e)
        }
    });

    orders
        .stream(streams::interval(1000, || Msg::OnTick))
        .stream(streams::window_event(Ev::KeyUp, |event| {
            let key_event: web_sys::KeyboardEvent = event.unchecked_into();
            match key_event.key().as_ref() {
                ESCAPE_KEY => Some(Msg::EscapeKey),
                _ => None,
            }
        }))
        .subscribe(Msg::UrlChanged);
    let tasks = match LocalStorage::get(TASKS_STORAGE_KEY) {
        Ok(tasks) => tasks,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => panic!(err),
        Err(_) => HashMap::new(),
    };
    let filters = match LocalStorage::get(FILTERS_STORAGE_KEY) {
        Ok(filters) => filters,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => panic!(err),
        Err(_) => Filters::default(),
    };
    let selected_task = url
        .search()
        .get(VIEW_TASK_SEARCH_KEY)
        .map(|v| {
            uuid::Uuid::parse_str(&v.first().unwrap_or(&String::new()))
                .map(|uuid| {
                    if tasks.contains_key(&uuid) {
                        Some(uuid)
                    } else {
                        None
                    }
                })
                .unwrap_or(None)
        })
        .flatten();
    if selected_task.is_none() {
        url.clone()
            .set_search(UrlSearch::default())
            .go_and_replace();
    }
    Model {
        tasks,
        selected_task,
        filters,
        base_url: url.to_base_url(),
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
struct Model {
    tasks: HashMap<uuid::Uuid, Task>,
    selected_task: Option<uuid::Uuid>,
    filters: Filters,
    base_url: Url,
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    pub fn home(self) -> Url {
        self.base_url().set_search(UrlSearch::default())
    }

    pub fn view_task(self, uuid: &uuid::Uuid) -> Url {
        self.base_url().set_search(UrlSearch::new(vec![(
            VIEW_TASK_SEARCH_KEY,
            vec![uuid.to_string()],
        )]))
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
    SelectedTaskPriorityChanged(String),
    SelectedTaskNotesChanged(String),
    CreateTask,
    DeleteSelectedTask,
    CompleteSelectedTask,
    StartSelectedTask,
    StopSelectedTask,
    MoveSelectedTaskToPending,
    OnTick,
    FiltersStatusTogglePending,
    FiltersStatusToggleDeleted,
    FiltersStatusToggleCompleted,
    FiltersStatusToggleWaiting,
    FiltersPriorityToggleNone,
    FiltersPriorityToggleLow,
    FiltersPriorityToggleMedium,
    FiltersPriorityToggleHigh,
    FiltersProjectChanged(String),
    FiltersTagsChanged(String),
    FiltersDescriptionChanged(String),
    FiltersReset,
    UrlChanged(subs::UrlChanged),
    EscapeKey,
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            Urls::new(&model.base_url).home().go_and_push();
            model.selected_task = None
        }
        Msg::SelectTask(Some(uuid)) => {
            Urls::new(&model.base_url).view_task(&uuid).go_and_push();
            model.selected_task = Some(uuid)
        }
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
        Msg::SelectedTaskPriorityChanged(new_priority) => {
            if let Some(uuid) = model.selected_task {
                if let Some(task) = &mut model.tasks.get_mut(&uuid) {
                    task.set_priority(match Priority::try_from(new_priority) {
                        Ok(p) => Some(p),
                        Err(()) => None,
                    });
                }
            }
        }
        Msg::SelectedTaskNotesChanged(new_notes) => {
            if let Some(uuid) = model.selected_task {
                if let Some(task) = &mut model.tasks.get_mut(&uuid) {
                    task.set_notes(new_notes)
                }
            }
        }
        Msg::CreateTask => {
            let task = Task::new();
            let id = task.uuid();
            model.tasks.insert(task.uuid(), task);
            model.selected_task = Some(id);
            Urls::new(&model.base_url).view_task(&id).go_and_push();
        }
        Msg::DeleteSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                Urls::new(&model.base_url).home().go_and_push();
                if let Some(task) = model.tasks.remove(&uuid) {
                    match task {
                        Task::Pending(_) | Task::Completed(_) | Task::Waiting(_) => {
                            model.tasks.insert(task.uuid(), task.delete());
                        }
                        Task::Deleted(_) => match window().confirm_with_message(
                            "Are you sure you want to permanently delete this task?",
                        ) {
                            Ok(true) => { /* already removed from set so just don't add it back */ }
                            Ok(false) | Err(_) => {
                                model.tasks.insert(task.uuid(), task);
                            }
                        },
                    }
                }
            }
        }
        Msg::CompleteSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                Urls::new(&model.base_url).home().go_and_push();
                if let Some(task) = model.tasks.remove(&uuid) {
                    model.tasks.insert(task.uuid(), task.complete());
                }
            }
        }
        Msg::StartSelectedTask => {
            if let Some(uuid) = model.selected_task.take() {
                Urls::new(&model.base_url).home().go_and_push();
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
                Urls::new(&model.base_url).home().go_and_push();
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
        Msg::MoveSelectedTaskToPending => {
            if let Some(uuid) = model.selected_task.take() {
                Urls::new(&model.base_url).home().go_and_push();
                if let Some(task) = model.tasks.remove(&uuid) {
                    match task {
                        Task::Pending(_) | Task::Waiting(_) => {
                            model.tasks.insert(task.uuid(), task);
                        }
                        Task::Deleted(task) => {
                            model
                                .tasks
                                .insert(task.uuid(), Task::Pending(task.undelete()));
                        }
                        Task::Completed(task) => {
                            model
                                .tasks
                                .insert(task.uuid(), Task::Pending(task.uncomplete()));
                        }
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
        Msg::FiltersPriorityToggleNone => {
            model.filters.priority_none = !model.filters.priority_none
        }
        Msg::FiltersPriorityToggleLow => model.filters.priority_low = !model.filters.priority_low,
        Msg::FiltersPriorityToggleMedium => {
            model.filters.priority_medium = !model.filters.priority_medium
        }
        Msg::FiltersPriorityToggleHigh => {
            model.filters.priority_high = !model.filters.priority_high
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
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let selected_task = url
                .search()
                .get(VIEW_TASK_SEARCH_KEY)
                .map(|v| {
                    uuid::Uuid::parse_str(&v.first().unwrap_or(&String::new()))
                        .map(|uuid| {
                            if model.tasks.contains_key(&uuid) {
                                Some(uuid)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                })
                .flatten();
            if selected_task.is_none() {
                url.set_search(UrlSearch::default()).go_and_replace();
            }
            model.selected_task = selected_task
        }
        Msg::EscapeKey => {
            if model.selected_task.is_some() {
                Urls::new(&model.base_url).home().go_and_push();
                model.selected_task = None
            }
        }
    }
    LocalStorage::insert(TASKS_STORAGE_KEY, &model.tasks).expect("save tasks to LocalStorage");
    LocalStorage::insert(FILTERS_STORAGE_KEY, &model.filters)
        .expect("save filters to LocalStorage");
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
            C![
                "bg-gray-200",
                "py-2",
                "px-4",
                "mr-8",
                "hover:bg-gray-300",
                "text-lg"
            ],
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
        view_text_input(
            "Description",
            &task.description(),
            true,
            Msg::SelectedTaskDescriptionChanged
        ),
        view_text_input(
            "Project",
            &task.project().join("."),
            false,
            Msg::SelectedTaskProjectChanged
        ),
        view_text_input(
            "Tags",
            &task.tags().join(" "),
            false,
            Msg::SelectedTaskTagsChanged
        ),
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Priority"],
            div![
                C!["flex", "flex-row"],
                select![
                    C!["border", "bg-white"],
                    option![
                        attrs! {
                            At::Value => "",
                            At::Selected => if task.priority().is_none() {
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        "None"
                    ],
                    option![
                        attrs! {
                            At::Value => "L",
                            At::Selected => if let Some(Priority::Low) = task.priority() {
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        "Low"
                    ],
                    option![
                        attrs! {
                            At::Value => "M",
                            At::Selected => if let Some(Priority::Medium)  = task.priority() {
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        "Medium"
                    ],
                    option![
                        attrs! {
                            At::Value => "H",
                            At::Selected => if let Some(Priority::High) = task.priority() {
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        "High"
                    ],
                    input_ev(Ev::Input, Msg::SelectedTaskPriorityChanged)
                ],
            ]
        ],
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Notes"],
            div![
                C!["flex", "flex-row"],
                textarea![
                    C!["flex-grow", "border", "mr-2"],
                    attrs! {
                        At::Value => task.notes(),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskNotesChanged)
                ],
                if task.notes().is_empty() {
                    pre![" "]
                } else {
                    button![
                        mouse_ev(Ev::Click, |_| Msg::SelectedTaskNotesChanged(String::new())),
                        div![C!["text-red-600"], "X"]
                    ]
                }
            ]
        ],
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
            IF!(matches!(task, Task::Pending(_)|Task::Waiting(_)) =>
                div![C!["mr-4"], view_button("Delete", Msg::DeleteSelectedTask)]
            ),
            IF!(matches!(task, Task::Deleted(_)) =>
                div![C!["mr-4"], view_button("Permanently delete", Msg::DeleteSelectedTask)]
            ),
            IF!(matches!(task, Task::Deleted(_)) =>
                div![C!["mr-4"], view_button("Undelete", Msg::MoveSelectedTaskToPending)]
            ),
            IF!(matches!(task, Task::Completed(_)) =>
                div![C!["mr-4"], view_button("Uncomplete", Msg::MoveSelectedTaskToPending)]
            ),
            view_button("Close", Msg::SelectTask(None))
        ]
    ]
}

fn view_text_input(
    name: &str,
    value: &str,
    autofocus: bool,
    f: impl FnOnce(String) -> Msg + Clone + 'static,
) -> Node<Msg> {
    let also_f = f.clone();
    div![
        C!["flex", "flex-col", "px-2", "mb-2"],
        div![C!["font-bold"], name],
        div![
            C!["flex", "flex-row"],
            input![
                C!["flex-grow", "border", "mr-2"],
                attrs! {
                    At::Value => value,
                    At::AutoFocus => if autofocus { AtValue::None } else { AtValue::Ignored }
                },
                input_ev(Ev::Input, f)
            ],
            if value.is_empty() {
                pre![" "]
            } else {
                button![
                    mouse_ev(Ev::Click, |_| also_f(String::new())),
                    div![C!["text-red-600"], "X"]
                ]
            }
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
        C!["flex", "flex-row", "flex-wrap", "justify-around"],
        view_filters(&model.filters),
        view_button("Create", Msg::CreateTask)
    ]
}

fn view_filters(filters: &Filters) -> Node<Msg> {
    div![
        C![
            "flex",
            "flex-row",
            "flex-wrap",
            "items-center",
            "bg-gray-50",
            "py-2",
            "px-2",
            "mx-2",
            "mb-2"
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
        div![
            C!["flex", "flex-col", "mr-8"],
            h2![C!["font-bold"], "Priority"],
            view_checkbox(
                "filters-priority-none",
                "None",
                filters.priority_none,
                Msg::FiltersPriorityToggleNone
            ),
            view_checkbox(
                "filters-priority-low",
                "Low",
                filters.priority_low,
                Msg::FiltersPriorityToggleLow
            ),
            view_checkbox(
                "filters-priority-medium",
                "Medium",
                filters.priority_medium,
                Msg::FiltersPriorityToggleMedium
            ),
            view_checkbox(
                "filters-priority-high",
                "High",
                filters.priority_high,
                Msg::FiltersPriorityToggleHigh
            ),
        ],
        view_text_input(
            "Description",
            &filters.description,
            false,
            Msg::FiltersDescriptionChanged
        ),
        view_text_input(
            "Project",
            &filters.project.join("."),
            false,
            Msg::FiltersProjectChanged
        ),
        view_text_input(
            "Tags",
            &filters.tags.join(" "),
            false,
            Msg::FiltersTagsChanged
        ),
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
        .values()
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
            priority: t.priority().to_owned(),
            active: match t {
                Task::Pending(t) => t.start().is_some(),
                Task::Completed(_) | Task::Deleted(_) | Task::Waiting(_) => false,
            },
            end: match t {
                Task::Pending(_) | Task::Waiting(_) => None,
                Task::Completed(t) => Some(*t.end()),
                Task::Deleted(t) => Some(*t.end()),
            },
        })
        .collect();

    // reverse sort so we have most urgent at the top
    tasks.sort_by(|t1, t2| match (t1.urgency, t2.urgency) {
        (Some(u1), Some(u2)) => u2.partial_cmp(&u1).unwrap(),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => t2.end.cmp(&t1.end),
    });
    let show_status = tasks
        .iter()
        .map(|t| &t.status)
        .collect::<HashSet<_>>()
        .len()
        > 1;
    let show_project = tasks.iter().any(|t| !t.project.is_empty());
    let show_tags = tasks.iter().any(|t| !t.tags.is_empty());
    let show_priority = tasks.iter().any(|t| t.priority.is_some());
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
                IF!(show_priority => th![C!["border-l-2"], "Priority"]),
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
                        IF!(t.urgency.unwrap_or(0.) > 5. => "bg-yellow-200"),
                        IF!(t.urgency.unwrap_or(0.) > 5. => "hover:bg-yellow-400"),
                        IF!(t.urgency.unwrap_or(0.) > 10. => "bg-red-200"),
                        IF!(t.urgency.unwrap_or(0.) > 10. => "hover:bg-red-400"),
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
                    IF!(show_priority => td![
                        C!["border-l-2", "text-center", "px-2"],
                        if let Some(p) = t.priority {
                            plain!(match p {
                                Priority::Low => "L",
                                Priority::Medium => "M",
                                Priority::High => "H",
                            })
                        } else {
                            empty![]
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
    priority: Option<Priority>,
    urgency: Option<f64>,
    end: Option<chrono::DateTime<chrono::Utc>>,
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
