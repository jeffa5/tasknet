use std::collections::{BTreeSet, HashMap, HashSet};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_checkbox, view_text_input},
    task::{Priority, Status, Task},
    urgency, Filters, GlobalModel, Msg as GMsg,
};

const FILTERS_STORAGE_KEY: &str = "tasknet-filters";
const CONTEXTS_STORAGE_KEY: &str = "tasknet-contexts";

pub fn init() -> Model {
    let filters = match LocalStorage::get(FILTERS_STORAGE_KEY) {
        Ok(filters) => filters,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse filters: {:?}", err)
        }
        Err(_) => Filters::default(),
    };
    let contexts = match LocalStorage::get(CONTEXTS_STORAGE_KEY) {
        Ok(contexts) => contexts,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse filters: {:?}", err)
        }
        Err(_) => HashMap::new(),
    };
    Model { filters, contexts }
}

#[derive(Debug)]
pub struct Model {
    filters: Filters,
    contexts: HashMap<String, Filters>,
}

#[derive(Clone)]
#[allow(clippy::pub_enum_variant_names)]
pub enum Msg {
    FiltersStatusTogglePending,
    FiltersStatusToggleDeleted,
    FiltersStatusToggleCompleted,
    FiltersStatusToggleWaiting,
    FiltersStatusToggleRecurring,
    FiltersPriorityToggleNone,
    FiltersPriorityToggleLow,
    FiltersPriorityToggleMedium,
    FiltersPriorityToggleHigh,
    FiltersProjectChanged(String),
    FiltersTagsChanged(String),
    FiltersDescriptionChanged(String),
    FiltersReset,
    FiltersSave,
    SelectedContextChanged(String),
}

pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<GMsg>) {
    match msg {
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
        Msg::FiltersStatusToggleRecurring => {
            model.filters.status_recurring = !model.filters.status_recurring
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
                new_project
                    .split('.')
                    .map(std::borrow::ToOwned::to_owned)
                    .collect()
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
            model.filters.description_and_notes = new_description
        }
        Msg::FiltersReset => model.filters = Filters::default(),
        Msg::FiltersSave => {
            match window().prompt_with_message("Name for the context (saved filters)") {
                Ok(Some(name)) => {
                    if !name.is_empty() {
                        if name.to_lowercase() == "custom" {
                            window()
                                .alert_with_message(&format!(
                                    "Cannot use name '{}' for context",
                                    name
                                ))
                                .unwrap_or_else(|e| log!(e))
                        } else {
                            let mut dups = Vec::new();
                            for (name, filters) in model.contexts.iter() {
                                if filters == &model.filters {
                                    dups.push(name.clone())
                                }
                            }
                            for name in dups {
                                model.contexts.remove(&name);
                            }
                            model.contexts.insert(name, model.filters.clone());
                            // TODO: check for matching values
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    log!(e);
                    window()
                        .alert_with_message("Failed to save filters")
                        .unwrap_or_else(|e| log!(e));
                }
            }
        }
        Msg::SelectedContextChanged(c) => {
            if let Some(filters) = model.contexts.get(&c) {
                model.filters = filters.clone()
            }
        }
    }
    LocalStorage::insert(FILTERS_STORAGE_KEY, &model.filters)
        .expect("save filters to LocalStorage");
    LocalStorage::insert(CONTEXTS_STORAGE_KEY, &model.contexts)
        .expect("save contexts to LocalStorage");
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![
        view_filters(model, &global_model.tasks),
        view_tasks(&global_model.tasks, &model.filters),
    ]
}

#[allow(clippy::too_many_lines)]
fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>, filters: &Filters) -> Node<GMsg> {
    let mut tasks: Vec<_> = tasks
        .values()
        .filter_map(|t| {
            if filters.filter_task(t) {
                Some(ViewableTask {
                    age: duration_string(
                        (chrono::offset::Utc::now()).signed_duration_since(*t.entry()),
                    ),
                    status: match t.status() {
                        Status::Pending => "Pending".to_owned(),
                        Status::Completed => "Completed".to_owned(),
                        Status::Deleted => "Deleted".to_owned(),
                        Status::Waiting => "Waiting".to_owned(),
                        Status::Recurring => "Recurring".to_owned(),
                    },
                    project: t.project().to_owned(),
                    description: t.description().to_owned(),
                    urgency: urgency::calculate(t),
                    uuid: t.uuid(),
                    tags: t.tags().to_owned(),
                    priority: t.priority().to_owned(),
                    active: t.start().is_some(),
                    end: t.end().to_owned(),
                    due: t.due().to_owned(),
                    scheduled: t.scheduled().to_owned(),
                })
            } else {
                None
            }
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
    let show_due = tasks.iter().any(|t| t.due.is_some());
    let show_scheduled = tasks.iter().any(|t| t.scheduled.is_some());
    div![
        C!["mt-2", "px-2", "pb-2"],
        table![
            C!["table-auto", "w-full"],
            tr![
                C!["border-b-2"],
                th!["Age"],
                IF!(show_due => th![C!["border-l-2"], "Due"]),
                IF!(show_scheduled => th![C!["border-l-2"], "Scheduled"]),
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
                        "cursor-pointer",
                        "select-none",
                        if t.active {
                            vec!["bg-green-200", "hover:bg-green-400"]
                        } else if is_next {
                            vec!["bg-blue-200", "hover:bg-blue-400"]
                        } else if t.urgency.unwrap_or(0.) > 10. {
                            vec!["bg-red-300", "hover:bg-red-400"]
                        } else if t.urgency.unwrap_or(0.) > 5. {
                            vec!["bg-yellow-200", "hover:bg-yellow-400"]
                        } else if i % 2 == 0 {
                            vec!["bg-gray-100", "hover:bg-gray-200"]
                        } else {
                            vec!["hover:bg-gray-200"]
                        }
                    ],
                    mouse_ev(Ev::Click, move |_| { GMsg::SelectTask(Some(id)) }),
                    td![C!["text-center", "px-2"], t.age.clone()],
                    IF!(show_due => td![C!["border-l-2", "text-center", "px-2"], t.due.map(|due|duration_string(due.signed_duration_since(chrono::offset::Utc::now())))]),
                    IF!(show_scheduled => td![C!["border-l-2", "text-center", "px-2"], t.scheduled.map(|scheduled|duration_string(scheduled.signed_duration_since(chrono::offset::Utc::now())))]),
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
    due: Option<chrono::DateTime<chrono::Utc>>,
    scheduled: Option<chrono::DateTime<chrono::Utc>>,
}

#[allow(clippy::too_many_lines)]
fn view_filters(model: &Model, tasks: &HashMap<uuid::Uuid, Task>) -> Node<GMsg> {
    let no_context_match = !model
        .contexts
        .values()
        .any(|filters| filters == &model.filters);
    div![
        C![
            "flex",
            "flex-row",
            "flex-wrap",
            "justify-center",
            "items-center",
            "bg-gray-100",
            "p-2",
            "mx-2",
        ],
        div![
            C!["flex", "flex-col", "mr-8"],
            h2![C!["font-bold"], "Status"],
            view_checkbox(
                "filters-status-pending",
                "Pending",
                model.filters.status_pending,
                GMsg::Home(Msg::FiltersStatusTogglePending)
            ),
            view_checkbox(
                "filters-status-deleted",
                "Deleted",
                model.filters.status_deleted,
                GMsg::Home(Msg::FiltersStatusToggleDeleted)
            ),
            view_checkbox(
                "filters-status-completed",
                "Completed",
                model.filters.status_completed,
                GMsg::Home(Msg::FiltersStatusToggleCompleted)
            ),
            view_checkbox(
                "filters-status-waiting",
                "Waiting",
                model.filters.status_waiting,
                GMsg::Home(Msg::FiltersStatusToggleWaiting)
            ),
            view_checkbox(
                "filters-status-recurring",
                "Recurring",
                model.filters.status_recurring,
                GMsg::Home(Msg::FiltersStatusToggleRecurring)
            ),
        ],
        div![
            C!["flex", "flex-col", "mr-8"],
            h2![C!["font-bold"], "Priority"],
            view_checkbox(
                "filters-priority-none",
                "None",
                model.filters.priority_none,
                GMsg::Home(Msg::FiltersPriorityToggleNone)
            ),
            view_checkbox(
                "filters-priority-low",
                "Low",
                model.filters.priority_low,
                GMsg::Home(Msg::FiltersPriorityToggleLow)
            ),
            view_checkbox(
                "filters-priority-medium",
                "Medium",
                model.filters.priority_medium,
                GMsg::Home(Msg::FiltersPriorityToggleMedium)
            ),
            view_checkbox(
                "filters-priority-high",
                "High",
                model.filters.priority_high,
                GMsg::Home(Msg::FiltersPriorityToggleHigh)
            ),
        ],
        view_text_input(
            "Description & Notes",
            &model.filters.description_and_notes,
            false,
            BTreeSet::new(),
            |s| GMsg::Home(Msg::FiltersDescriptionChanged(s))
        ),
        view_text_input(
            "Project",
            &model.filters.project.join("."),
            false,
            BTreeSet::new(),
            |s| GMsg::Home(Msg::FiltersProjectChanged(s))
        ),
        view_text_input(
            "Tags",
            &model.filters.tags.join(" "),
            false,
            BTreeSet::new(),
            |s| GMsg::Home(Msg::FiltersTagsChanged(s))
        ),
        div![
            C!["mr-2"],
            tasks
                .values()
                .filter(|t| model.filters.filter_task(t))
                .count(),
            "/",
            tasks.len()
        ],
        div![
            C!["flex", "flex-col"],
            view_button("Reset Filters", GMsg::Home(Msg::FiltersReset)),
            view_button("Save to context", GMsg::Home(Msg::FiltersSave)),
        ],
        div![
            C!["flex", "flex-col"],
            div![C!["font-bold"], "Context"],
            select![
                C!["border", "bg-white"],
                option![
                    attrs! {
                        At::Value => "custom",
                        At::Selected => if no_context_match {
                            AtValue::None
                        }else {
                            AtValue::Ignored
                        }
                    },
                    "Custom"
                ],
                model
                    .contexts
                    .iter()
                    .map(|(name, filters)| option![
                        attrs! {
                            At::Value => name,
                            At::Selected => if filters == &model.filters{
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        name
                    ])
                    .collect::<Vec<_>>(),
                input_ev(Ev::Input, |s| GMsg::Home(Msg::SelectedContextChanged(s)))
            ],
        ]
    ]
}
