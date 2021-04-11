use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_checkbox, view_text_input},
    task::{Id, Priority, Status, Task},
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
    Model {
        filters,
        contexts,
        sort_field: Field::Urgency,
        sort_direction: SortDirection::Descending,
    }
}

#[derive(Debug)]
pub struct Model {
    filters: Filters,
    contexts: HashMap<String, Filters>,
    sort_field: Field,
    sort_direction: SortDirection,
}

#[derive(Debug)]
enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Age,
    Due,
    Scheduled,
    Project,
    Tags,
    Priority,
    Description,
    Status,
    Urgency,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Age => "Age",
                Self::Due => "Due",
                Self::Scheduled => "Scheduled",
                Self::Project => "Project",
                Self::Tags => "Tags",
                Self::Priority => "Priority",
                Self::Description => "Description",
                Self::Status => "Status",
                Self::Urgency => "Urgency",
            }
        )
    }
}

#[derive(Debug, Clone)]
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
    ToggleShowFilters,
    SelectedContextChanged(String),
    ContextsRemove,
    ToggleSort(Field),
}

#[allow(clippy::too_many_lines)]
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
                            let current_filters = model.filters.clone();
                            model
                                .contexts
                                .retain(|_, filters| filters != &current_filters);
                            model.contexts.insert(name, model.filters.clone());
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
        Msg::ToggleShowFilters => {
            model.filters.show = !model.filters.show;
        }
        Msg::SelectedContextChanged(c) => {
            if let Some(filters) = model.contexts.get(&c) {
                model.filters = filters.clone()
            }
        }
        Msg::ContextsRemove => {
            let current_filters = model.filters.clone();
            model
                .contexts
                .retain(|_, filters| filters != &current_filters);
        }
        Msg::ToggleSort(field) => {
            model.sort_field = field;
            match model.sort_direction {
                SortDirection::Ascending => model.sort_direction = SortDirection::Descending,
                SortDirection::Descending => model.sort_direction = SortDirection::Ascending,
            }
        }
    }
    LocalStorage::insert(FILTERS_STORAGE_KEY, &model.filters)
        .expect("save filters to LocalStorage");
    LocalStorage::insert(CONTEXTS_STORAGE_KEY, &model.contexts)
        .expect("save contexts to LocalStorage");
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let tasks = global_model.document.tasks();
    div![
        if model.filters.show {
            view_filters(model, &tasks)
        } else {
            empty!()
        },
        view_tasks(&tasks, model)
    ]
}

#[allow(clippy::too_many_lines)]
fn view_tasks(tasks: &HashMap<Id, Task>, model: &Model) -> Node<GMsg> {
    let tasks: Vec<_> = tasks
        .iter()
        .filter(|(_, t)| model.filters.filter_task(t))
        .collect();

    if tasks.is_empty() {
        div![
            C![
                "flex",
                "flex-col",
                "flex-wrap",
                "items-center",
                "font-bold",
                "m-2"
            ],
            span!["No tasks to show, either create a task or change some filters."]
        ]
    } else {
        view_tasks_table(tasks, model)
    }
}

fn view_tasks_table(mut tasks: Vec<(&Id, &Task)>, model: &Model) -> Node<GMsg> {
    tasks.sort_by(|(_, t1), (_, t2)| sort_viewable_task(model.sort_field, t1, t2));
    if matches!(model.sort_direction, SortDirection::Descending) {
        tasks.reverse();
    }

    let tasks = tasks
        .iter()
        .map(|(uuid, t)| ViewableTask {
            age: duration_string((chrono::offset::Utc::now()).signed_duration_since(**t.entry())),
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
            uuid: ***uuid,
            tags: t.tags().to_owned(),
            priority: t.priority().to_owned(),
            active: t.start().is_some(),
            due: t.due().as_ref().map(|d| **d).to_owned(),
            scheduled: t.scheduled().as_ref().map(|d| **d).to_owned(),
        })
        .collect::<Vec<_>>();

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
                th![view_sort_button(model, Field::Age)],
                IF!(show_due => th![C!["border-l-2"], view_sort_button(model, Field::Due)]),
                IF!(show_scheduled => th![C!["border-l-2"], view_sort_button(model, Field::Scheduled)]),
                IF!(show_status => th![C!["border-l-2"], view_sort_button(model, Field::Status)]),
                IF!(show_project => th![C!["border-l-2"], view_sort_button(model, Field::Project)]),
                IF!(show_tags => th![C!["border-l-2"], view_sort_button(model, Field::Tags)]),
                IF!(show_priority => th![C!["border-l-2"], view_sort_button(model, Field::Priority)]),
                th![C!["border-l-2"], view_sort_button(model, Field::Description)],
                th![C!["border-l-2"],
                    view_sort_button(model, Field::Urgency)
                ]
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

fn view_sort_button(model: &Model, field: Field) -> Node<GMsg> {
    button![
        C!["w-full"],
        mouse_ev(Ev::Click, move |_| GMsg::Home(Msg::ToggleSort(field))),
        format!(
            "{}{}",
            field,
            if model.sort_field == field {
                match model.sort_direction {
                    SortDirection::Ascending => "\u{2b06}",  // ⬆
                    SortDirection::Descending => "\u{2b07}", // ⬇
                }
            } else {
                "\u{2b0d}" // ⬍
            }
        ),
    ]
}

fn sort_viewable_task(sort_field: Field, t1: &Task, t2: &Task) -> Ordering {
    match sort_field {
        Field::Urgency => match (urgency::calculate(t1), urgency::calculate(t2)) {
            (Some(u1), Some(u2)) => u1.partial_cmp(&u2).unwrap(),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => t2.entry().cmp(t1.entry()),
        },
        Field::Age => t2.entry().cmp(t1.entry()),
        Field::Due => match (t1.due(), t2.due()) {
            (Some(d1), Some(d2)) => d1.cmp(d2),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => t2.entry().cmp(t1.entry()),
        },
        Field::Scheduled => match (t1.scheduled(), t2.scheduled()) {
            (Some(d1), Some(d2)) => d1.cmp(d2),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => t2.entry().cmp(t1.entry()),
        },
        Field::Project => t1.project().cmp(t2.project()),
        Field::Tags => match (t1.tags(), t2.tags()) {
            ([], []) => t2.entry().cmp(t1.entry()),
            ([], [..]) => Ordering::Greater,
            ([..], []) => Ordering::Less,
            (t1, t2) => t1.cmp(t2),
        },
        Field::Priority => match (t1.priority(), t2.priority()) {
            (Some(d1), Some(d2)) => d1.cmp(d2),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => t2.entry().cmp(t1.entry()),
        },
        Field::Description => t1.description().cmp(t2.description()),
        Field::Status => t1.status().cmp(t2.status()),
    }
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
    due: Option<chrono::DateTime<chrono::Utc>>,
    scheduled: Option<chrono::DateTime<chrono::Utc>>,
}

#[allow(clippy::too_many_lines)]
fn view_filters(model: &Model, tasks: &HashMap<Id, Task>) -> Node<GMsg> {
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
            view_button("Remove context", GMsg::Home(Msg::ContextsRemove)),
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
