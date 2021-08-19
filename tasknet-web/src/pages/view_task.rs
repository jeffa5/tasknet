use std::{collections::BTreeSet, convert::TryFrom};

use chrono::{Datelike, Timelike};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_text_input},
    task::{DateTime, Priority, RecurUnit, Status, Task},
    GlobalModel, Msg as GMsg, Recur, Urls,
};

const ESCAPE_KEY: &str = "Escape";

pub fn init(uuid: uuid::Uuid, task: Task, orders: &mut impl Orders<GMsg>) -> Model {
    orders.stream(streams::window_event(Ev::KeyUp, |event| {
        let key_event: web_sys::KeyboardEvent = event.unchecked_into();
        match key_event.key().as_ref() {
            ESCAPE_KEY => Some(GMsg::SelectTask(None)),
            _ => None,
        }
    }));
    Model {
        task_id: uuid,
        task,
    }
}

#[derive(Debug)]
pub struct Model {
    pub task_id: uuid::Uuid,
    pub task: Task,
}

#[derive(Debug, Clone)]
pub enum Msg {
    SelectedTaskDescriptionChanged(String),
    SelectedTaskProjectChanged(String),
    SelectedTaskTagsChanged(String),
    SelectedTaskPriorityChanged(String),
    SelectedTaskNotesChanged(String),
    SelectedTaskDueDateChanged(String),
    SelectedTaskDueTimeChanged(String),
    SelectedTaskScheduledDateChanged(String),
    SelectedTaskScheduledTimeChanged(String),
    SelectedTaskRecurUnitChanged(String),
    SelectedTaskRecurAmountChanged(String),
    DeleteSelectedTask,
    CompleteSelectedTask,
    StartSelectedTask,
    StopSelectedTask,
    MoveSelectedTaskToPending,
    SaveCloseTask,
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub fn update(
    msg: Msg,
    global_model: &mut GlobalModel,
    model: &mut Model,
    orders: &mut impl Orders<GMsg>,
) {
    match msg {
        Msg::SelectedTaskDescriptionChanged(new_description) => {
            model.task.set_description(new_description)
        }
        Msg::SelectedTaskProjectChanged(new_project) => {
            let new_project = new_project.trim();
            model.task.set_project(if new_project.is_empty() {
                Vec::new()
            } else {
                new_project
                    .split('.')
                    .map(std::borrow::ToOwned::to_owned)
                    .collect()
            });
        }
        Msg::SelectedTaskTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            model.task.set_tags(if new_tags.is_empty() {
                Vec::new()
            } else {
                let mut tags: Vec<_> = new_tags
                    .split_whitespace()
                    .map(std::borrow::ToOwned::to_owned)
                    .collect();
                if new_end {
                    tags.push(String::new())
                }
                tags
            })
        }
        Msg::SelectedTaskPriorityChanged(new_priority) => model
            .task
            .set_priority(Priority::try_from(new_priority).ok()),
        Msg::SelectedTaskNotesChanged(new_notes) => model.task.set_notes(new_notes),
        Msg::SelectedTaskDueDateChanged(new_date) => {
            let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
            match new_date {
                Ok(new_date) => {
                    let due = model.task.due();
                    match due {
                        None => model.task.set_due(Some(DateTime(chrono::DateTime::from_utc(
                            new_date.and_hms(0, 0, 0),
                            chrono::Utc,
                        )))),
                        Some(due) => {
                            let due = due
                                .with_year(new_date.year())
                                .and_then(|due| due.with_month(new_date.month()))
                                .and_then(|due| due.with_day(new_date.day()));
                            model.task.set_due(due.map(DateTime))
                        }
                    }
                }
                Err(_) => model.task.set_due(None),
            }
        }
        Msg::SelectedTaskDueTimeChanged(new_time) => {
            let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
            match new_time {
                Ok(new_time) => {
                    let due = model.task.due();
                    match due {
                        None => {
                            let now = chrono::offset::Utc::now();
                            let now = now
                                .with_hour(new_time.hour())
                                .and_then(|now| now.with_minute(new_time.minute()));
                            model.task.set_due(now.map(DateTime))
                        }
                        Some(due) => {
                            let due = due
                                .with_hour(new_time.hour())
                                .and_then(|due| due.with_minute(new_time.minute()));
                            model.task.set_due(due.map(DateTime))
                        }
                    }
                }
                Err(_) => model.task.set_due(None),
            }
        }
        Msg::SelectedTaskScheduledDateChanged(new_date) => {
            let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
            match new_date {
                Ok(new_date) => {
                    let scheduled = model.task.scheduled();
                    match scheduled {
                        None => {
                            model
                                .task
                                .set_scheduled(Some(DateTime(chrono::DateTime::from_utc(
                                    new_date.and_hms(0, 0, 0),
                                    chrono::Utc,
                                ))))
                        }
                        Some(scheduled) => {
                            let scheduled = scheduled
                                .with_year(new_date.year())
                                .and_then(|scheduled| scheduled.with_month(new_date.month()))
                                .and_then(|scheduled| scheduled.with_day(new_date.day()));
                            model.task.set_scheduled(scheduled.map(DateTime))
                        }
                    }
                }
                Err(_) => model.task.set_scheduled(None),
            }
        }
        Msg::SelectedTaskScheduledTimeChanged(new_time) => {
            let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
            match new_time {
                Ok(new_time) => {
                    let scheduled = model.task.scheduled();
                    match scheduled {
                        None => {
                            let now = chrono::offset::Utc::now();
                            let now = now
                                .with_hour(new_time.hour())
                                .and_then(|now| now.with_minute(new_time.minute()));
                            model.task.set_scheduled(now.map(DateTime))
                        }
                        Some(scheduled) => {
                            let scheduled = scheduled
                                .with_hour(new_time.hour())
                                .and_then(|scheduled| scheduled.with_minute(new_time.minute()));
                            model.task.set_scheduled(scheduled.map(DateTime))
                        }
                    }
                }
                Err(_) => model.task.set_scheduled(None),
            }
        }
        Msg::SelectedTaskRecurAmountChanged(new_amount) => {
            if let Ok(n) = new_amount.parse::<u16>() {
                if n > 0 {
                    model.task.set_recur(Some(Recur {
                        amount: n,
                        unit: model
                            .task
                            .recur()
                            .as_ref()
                            .map_or(RecurUnit::Week, |r| r.unit),
                    }))
                } else {
                    model.task.set_recur(None)
                }
            }
        }
        Msg::SelectedTaskRecurUnitChanged(new_unit) => match RecurUnit::try_from(new_unit) {
            Ok(unit) => model.task.set_recur(Some(Recur {
                amount: model.task.recur().as_ref().map_or(1, |r| r.amount),
                unit,
            })),
            Err(()) => model.task.set_recur(None),
        },
        Msg::DeleteSelectedTask => match model.task.status() {
            Status::Pending | Status::Completed | Status::Waiting | Status::Recurring => {
                model.task.delete();
                let msg = global_model
                    .document
                    .set_task(model.task_id, model.task.clone());
                if let Some(msg) = msg {
                    orders.send_msg(msg);
                }
                orders.send_msg(GMsg::SelectTask(None));
            }
            Status::Deleted => match window()
                .confirm_with_message("Are you sure you want to permanently delete this task?")
            {
                Ok(true) => {
                    orders.request_url(Urls::new(&global_model.base_url).home());
                    let msg = global_model.document.remove_task(model.task_id);
                    if let Some(msg) = msg {
                        orders.send_msg(msg);
                    }
                }
                Ok(false) | Err(_) => {}
            },
        },
        Msg::CompleteSelectedTask => {
            model.task.complete();
            let msg = global_model
                .document
                .set_task(model.task_id, model.task.clone());
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
            orders.send_msg(GMsg::SelectTask(None));
        }
        Msg::StartSelectedTask => {
            model.task.activate();
            let msg = global_model
                .document
                .set_task(model.task_id, model.task.clone());
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
            orders.send_msg(GMsg::SelectTask(None));
        }
        Msg::StopSelectedTask => {
            model.task.deactivate();
            let msg = global_model
                .document
                .set_task(model.task_id, model.task.clone());
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
            orders.send_msg(GMsg::SelectTask(None));
        }
        Msg::MoveSelectedTaskToPending => {
            model.task.restore();
            let msg = global_model
                .document
                .set_task(model.task_id, model.task.clone());
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
            orders.send_msg(GMsg::SelectTask(None));
        }
        Msg::SaveCloseTask => {
            let msg = global_model
                .document
                .set_task(model.task_id, model.task.clone());
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
            orders.send_msg(GMsg::SelectTask(None));
        }
    }
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![view_selected_task(global_model, model)]
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn view_selected_task(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let is_pending = matches!(model.task.status(), Status::Pending);
    let status_text = match model.task.status() {
        Status::Pending => "Pending",
        Status::Deleted => "Deleted",
        Status::Completed => "Completed",
        Status::Waiting => "Waiting",
        Status::Recurring => "Recurring",
    };
    let changes_text = if model.task
        == global_model
            .document
            .task(&model.task_id)
            .cloned()
            .unwrap_or_default()
    {
        span![C!["font-bold", "text-green-500"], "\u{2713} All saved"]
    } else {
        span![
            C!["font-bold", "text-yellow-500"],
            "\u{26A0} Unsaved changes"
        ]
    };
    let disable_changes = model.task.description().is_empty();
    let start = model.task.start();
    let end = model.task.end();
    let urgency = global_model.settings.urgency.calculate(&model.task);
    let active = model.task.start().is_some();
    let is_next = model.task.tags().contains(&"next".to_owned());
    let project_lowercase = model.task.project().join(".").to_lowercase();
    let project_suggestions = global_model
        .document
        .tasks()
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
    let mut tags_suggestions = global_model
        .document
        .tasks()
        .values()
        .flat_map(|saved_task| {
            saved_task
                .tags()
                .iter()
                .filter_map(|saved_tag| {
                    let input_tags = model.task.tags();
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
    if !is_next {
        tags_suggestions.insert("next".to_owned());
    }
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
            C!["flex", "flex-row", "justify-between", "pl-2"],
            div![span![C!["font-bold"], "Status: "], status_text],
            div![changes_text]
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
            model.task.entry().to_string()
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
            model.task.description(),
            "",
            true,
            BTreeSet::new(),
            |s| GMsg::ViewTask(Msg::SelectedTaskDescriptionChanged(s))
        ),
        view_text_input(
            "Project",
            &model.task.project().join("."),
            "",
            false,
            project_suggestions,
            |s| GMsg::ViewTask(Msg::SelectedTaskProjectChanged(s))
        ),
        div![
            C!["flex", "flex-col", "px-2", "mb-2"],
            div![C!["font-bold"], "Tags"],
            div![
                C!["flex", "flex-row"],
                input![
                    C!["flex-grow", "border", "mr-2"],
                    attrs! {
                        At::Value => model.task.tags().join(" "),
                        At::AutoFocus => AtValue::Ignored
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(Msg::SelectedTaskTagsChanged(
                        s
                    )))
                ],
                if model.task.tags().join(" ").is_empty() {
                    pre![" "]
                } else {
                    button![
                        mouse_ev(Ev::Click, |_| GMsg::ViewTask(Msg::SelectedTaskTagsChanged(
                            String::new()
                        ))),
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
                        let tags = model.task.tags().join(" ");
                        button![
                            C!["mr-2", "mt-2", "px-1", "bg-gray-200"],
                            mouse_ev(Ev::Click, move |_| {
                                if tags.ends_with(' ') || tags.is_empty() {
                                    GMsg::ViewTask(Msg::SelectedTaskTagsChanged(format!(
                                        "{} {}",
                                        tags, sug_clone
                                    )))
                                } else {
                                    let split_tags = tags.split_whitespace().collect::<Vec<_>>();
                                    let tags = split_tags
                                        .iter()
                                        .take(split_tags.len() - 1)
                                        .map(|s| s.to_owned())
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    GMsg::ViewTask(Msg::SelectedTaskTagsChanged(format!(
                                        "{} {}",
                                        tags, sug_clone
                                    )))
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
                            At::Selected => if model.task.priority().is_none() {
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
                            At::Selected => if let Some(Priority::Low) = model.task.priority() {
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
                            At::Selected => if let Some(Priority::Medium)  = model.task.priority() {
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
                            At::Selected => if let Some(Priority::High) = model.task.priority() {
                                AtValue::None
                            } else {
                                AtValue::Ignored
                            }
                        },
                        "High"
                    ],
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskPriorityChanged(s)
                    ))
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
                        At::Value => model.task.due().as_ref().map_or_else(String::new, |due| due.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => model.task.due().as_ref().map_or_else(String::new, |due| due.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueTimeChanged(s)
                    ))
                ],
                if let Some(due) = model.task.due() {
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
                        At::Value => model.task.scheduled().as_ref().map_or_else(String::new, |scheduled| scheduled.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => model.task.scheduled().as_ref().map_or_else(String::new, |scheduled| scheduled.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledTimeChanged(s)
                    ))
                ],
                if let Some(scheduled) = model.task.scheduled() {
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
                        At::Value => model.task.recur().as_ref().map_or(0, |r|r.amount),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskRecurAmountChanged(s)
                    ))
                ],
                span![" "],
                select![
                    C!["border", "bg-white"],
                    option![
                        attrs! {
                            At::Value => "",
                            At::Selected => if model.task.status() == &Status::Recurring {
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
                            At::Selected => model.task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
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
                            At::Selected => model.task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
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
                            At::Selected => model.task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
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
                            At::Selected => model.task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
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
                            At::Selected => model.task.recur().as_ref().map_or(AtValue::Ignored, |recur| {
                                if recur.unit == RecurUnit::Hour {
                                    AtValue::None
                                } else {
                                    AtValue::Ignored
                                }
                            })
                        },
                        "Hours"
                    ],
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskRecurUnitChanged(s)
                    ))
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
                        At::Value => model.task.notes(),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskNotesChanged(s)
                    ))
                ],
                if model.task.notes().is_empty() {
                    pre![" "]
                } else {
                    button![
                        mouse_ev(Ev::Click, |_| GMsg::ViewTask(
                            Msg::SelectedTaskNotesChanged(String::new())
                        )),
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
                        view_button("Stop", GMsg::ViewTask(Msg::StopSelectedTask),disable_changes)
                    } else {
                        view_button("Start", GMsg::ViewTask(Msg::StartSelectedTask),disable_changes)
                    }
                ]
            ),
            IF!(is_pending =>
                div![ view_button("Complete", GMsg::ViewTask(Msg::CompleteSelectedTask),disable_changes)]
            ),
            IF!(matches!(model.task.status(), Status::Pending|Status::Waiting|Status::Recurring) =>
                div![ view_button("Delete", GMsg::ViewTask(Msg::DeleteSelectedTask),disable_changes)]
            ),
            IF!(matches!(model.task.status(), Status::Deleted) =>
                div![ view_button("Permanently delete", GMsg::ViewTask(Msg::DeleteSelectedTask),disable_changes)]
            ),
            IF!(matches!(model.task.status(), Status::Deleted) =>
                div![ view_button("Restore", GMsg::ViewTask(Msg::MoveSelectedTaskToPending),disable_changes)]
            ),
            IF!(matches!(model.task.status(), Status::Completed) =>
                div![ view_button("Uncomplete", GMsg::ViewTask(Msg::MoveSelectedTaskToPending),disable_changes)]
            ),
            view_button(
                "Save & Close",
                GMsg::ViewTask(Msg::SaveCloseTask),
                disable_changes
            )
        ]
    ]
}
