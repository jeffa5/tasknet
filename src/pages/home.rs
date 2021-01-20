use std::collections::{BTreeSet, HashMap, HashSet};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_checkbox, view_text_input},
    task::{Priority, Status, Task},
    urgency, Filters, Model, Msg,
};

pub fn view(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "container", "mx-auto"],
        view_filters(&model.filters, &model.tasks),
        view_tasks(&model.tasks, &model.filters),
    ]
}

#[allow(clippy::too_many_lines)]
fn view_tasks(tasks: &HashMap<uuid::Uuid, Task>, filters: &Filters) -> Node<Msg> {
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
                    mouse_ev(Ev::Click, move |_| { Msg::SelectTask(Some(id)) }),
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
fn view_filters(filters: &Filters, tasks: &HashMap<uuid::Uuid, Task>) -> Node<Msg> {
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
            view_checkbox(
                "filters-status-recurring",
                "Recurring",
                filters.status_recurring,
                Msg::FiltersStatusToggleRecurring
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
            "Description & Notes",
            &filters.description_and_notes,
            false,
            BTreeSet::new(),
            Msg::FiltersDescriptionChanged
        ),
        view_text_input(
            "Project",
            &filters.project.join("."),
            false,
            BTreeSet::new(),
            Msg::FiltersProjectChanged
        ),
        view_text_input(
            "Tags",
            &filters.tags.join(" "),
            false,
            BTreeSet::new(),
            Msg::FiltersTagsChanged
        ),
        div![
            C!["mr-2"],
            tasks.values().filter(|t| filters.filter_task(t)).count(),
            "/",
            tasks.len()
        ],
        view_button("Reset Filters", Msg::FiltersReset),
    ]
}
