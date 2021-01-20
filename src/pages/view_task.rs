use std::collections::{BTreeSet, HashMap};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_text_input},
    task::{Priority, RecurUnit, Status, Task},
    urgency,
};

pub fn init(uuid: uuid::Uuid) -> Model {
    Model {
        selected_task: uuid,
    }
}

#[derive(Debug)]
pub struct Model {
    selected_task: uuid::Uuid,
}

pub enum Msg {}

pub fn view(model: &Model, task: &Task) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "container", "mx-auto"],
        view_selected_task(task, &model.tasks),
    ]
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn view_selected_task(task: &Task, tasks: &HashMap<uuid::Uuid, Task>) -> Node<Msg> {
    let is_pending = matches!(task.status(), Status::Pending);
    let start = task.start();
    let end = task.end();
    let urgency = urgency::calculate(task);
    let active = task.start().is_some();
    let is_next = task.tags().contains(&"next".to_owned());
    let project_lowercase = task.project().join(".").to_lowercase();
    let project_suggestions = tasks
        .values()
        .filter_map(|t| {
            let t_proj = t.project().join(".").to_lowercase();
            if t_proj.contains(&project_lowercase) && t_proj != project_lowercase {
                Some(t.project().join("."))
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    let mut tags_suggestions = tasks
        .values()
        .flat_map(|saved_task| {
            saved_task
                .tags()
                .iter()
                .filter_map(|saved_tag| {
                    let input_tags = task.tags();
                    if input_tags.is_empty()
                        || (!input_tags.contains(&saved_tag.to_lowercase())
                            && input_tags
                                .iter()
                                .any(|input_tag| saved_tag.contains(input_tag)))
                    {
                        Some(saved_tag.to_owned())
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
        })
        .collect::<BTreeSet<_>>();
    tags_suggestions.insert("next".to_owned());
    div![
        C![
            "flex",
            "flex-col",
            "bg-gray-100",
            "p-2",
            "border-4",
            if active {
                "border-green-200"
            } else if is_next {
                "border-blue-200"
            } else if urgency.unwrap_or(0.) > 10. {
                "border-red-200"
            } else if urgency.unwrap_or(0.) > 5. {
                "border-yellow-200"
            } else {
                "border-gray-200"
            }
        ],
        div![
            C!["pl-2"],
            span![C!["font-bold"], "Status: "],
            match task.status() {
                Status::Pending => "Pending",
                Status::Deleted => "Deleted",
                Status::Completed => "Completed",
                Status::Waiting => "Waiting",
                Status::Recurring => "Recurring",
            }
        ],
        if let Some(urgency) = urgency {
            div![
                C!["pl-2"],
                span![C!["font-bold"], "Urgency: "],
                plain![format!("{:.2}", urgency)]
            ]
        } else {
            empty![]
        },
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
            task.description(),
            true,
            BTreeSet::new(),
            Msg::SelectedTaskDescriptionChanged
        ),
        view_text_input(
            "Project",
            &task.project().join("."),
            false,
            project_suggestions,
            Msg::SelectedTaskProjectChanged
        ),
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Tags"],
            div![
                C!["flex", "flex-row"],
                input![
                    C!["flex-grow", "border", "mr-2"],
                    attrs! {
                        At::Value => task.tags().join(" "),
                        At::AutoFocus => AtValue::Ignored
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskTagsChanged)
                ],
                if task.tags().join(" ").is_empty() {
                    pre![" "]
                } else {
                    button![
                        mouse_ev(Ev::Click, |_| Msg::SelectedTaskTagsChanged(String::new())),
                        div![C!["text-red-600"], "X"]
                    ]
                }
            ],
            div![
                C!["flex", "flex-row", "overflow-hidden"],
                tags_suggestions
                    .into_iter()
                    .map(|sug| {
                        let sug_clone = sug.clone();
                        let tags = task.tags().join(" ");
                        button![
                            C!["mr-2", "mt-2", "px-1", "bg-gray-200"],
                            mouse_ev(Ev::Click, move |_| {
                                if tags.ends_with(' ') || tags.is_empty() {
                                    Msg::SelectedTaskTagsChanged(format!("{} {}", tags, sug_clone))
                                } else {
                                    let split_tags = tags.split_whitespace().collect::<Vec<_>>();
                                    let tags = split_tags
                                        .iter()
                                        .take(split_tags.len() - 1)
                                        .map(|s| s.to_owned())
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    Msg::SelectedTaskTagsChanged(format!("{} {}", tags, sug_clone))
                                }
                            }),
                            sug
                        ]
                    })
                    .collect::<Vec<_>>()
            ]
        ],
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
            div![C!["font-bold"], "Due"],
            div![
                C!["flex", "flex-row"],
                input![
                    C!["mr-4"],
                    attrs! {
                        At::Type => "date",
                        At::Value => task.due().map_or_else(String::new, |due| due.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskDueDateChanged)
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.due().map_or_else(String::new, |due| due.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskDueTimeChanged)
                ],
                if let Some(due) = task.due() {
                    span![
                        C!["ml-2"],
                        duration_string(due.signed_duration_since(chrono::offset::Utc::now()))
                    ]
                } else {
                    empty![]
                }
            ]
        ],
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Scheduled"],
            div![
                C!["flex", "flex-row"],
                input![
                    C!["mr-4"],
                    attrs! {
                        At::Type => "date",
                        At::Value => task.scheduled().map_or_else(String::new, |scheduled| scheduled.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskScheduledDateChanged)
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.scheduled().map_or_else(String::new, |scheduled| scheduled.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskScheduledTimeChanged)
                ],
                if let Some(scheduled) = task.scheduled() {
                    span![
                        C!["ml-2"],
                        duration_string(
                            scheduled.signed_duration_since(chrono::offset::Utc::now())
                        )
                    ]
                } else {
                    empty![]
                }
            ]
        ],
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Recur"],
            div![
                span!["Every "],
                input![
                    attrs! {
                        At::Type => "number",
                        At::Value => task.recur().as_ref().map_or(0, |r|r.amount),
                    },
                    input_ev(Ev::Input, Msg::SelectedTaskRecurAmountChanged)
                ],
                span![" "],
                select![
                    C!["border", "bg-white"],
                    option![
                        attrs! {
                            At::Value => "",
                            At::Selected => if task.status() == &Status::Recurring {
                                AtValue::Ignored
                            } else {
                                AtValue::None
                            }
                        },
                        "None"
                    ],
                    option![
                        attrs! {
                            At::Value => "Year",
                            At::Selected => task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Year {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Years"
                    ],
                    option![
                        attrs! {
                            At::Value => "Month",
                            At::Selected => task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Month {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Months"
                    ],
                    option![
                        attrs! {
                            At::Value => "Week",
                            At::Selected => task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Week {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Weeks"
                    ],
                    option![
                        attrs! {
                            At::Value => "Day",
                            At::Selected => task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Day {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Days"
                    ],
                    option![
                        attrs! {
                            At::Value => "Hour",
                            At::Selected => task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Hour {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Hours"
                    ],
                    input_ev(Ev::Input, Msg::SelectedTaskRecurUnitChanged)
                ]
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
                    if start.is_some() {
                        view_button("Stop", Msg::StopSelectedTask)
                    } else {
                        view_button("Start", Msg::StartSelectedTask)
                    }
                ]
            ),
            IF!(is_pending =>
                div![ view_button("Complete", Msg::CompleteSelectedTask)]
            ),
            IF!(matches!(task.status(), Status::Pending|Status::Waiting|Status::Recurring) =>
                div![ view_button("Delete", Msg::DeleteSelectedTask)]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button("Permanently delete", Msg::DeleteSelectedTask)]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button("Undelete", Msg::MoveSelectedTaskToPending)]
            ),
            IF!(matches!(task.status(), Status::Completed) =>
                div![ view_button("Uncomplete", Msg::MoveSelectedTaskToPending)]
            ),
            view_button("Close", Msg::SelectTask(None))
        ]
    ]
}
