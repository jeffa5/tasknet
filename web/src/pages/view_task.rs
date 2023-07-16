use std::{collections::BTreeSet, convert::TryFrom};

use chrono::{Datelike, Timelike};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button_str, view_text_input},
    document::Document,
    task::{DateTime, Priority, RecurUnit, Status, Task, TaskId},
    urgency, GlobalModel, Msg as GMsg, Recur, Urls,
};

const ESCAPE_KEY: &str = "Escape";

pub fn init(id: TaskId, orders: &mut impl Orders<GMsg>) -> Model {
    orders.stream(streams::window_event(Ev::KeyUp, |event| {
        let key_event: web_sys::KeyboardEvent = event.unchecked_into();
        match key_event.key().as_ref() {
            ESCAPE_KEY => Some(GMsg::ViewTask(Msg::EscapeKey)),
            _ => None,
        }
    }));
    Model { selected_task: id }
}

#[derive(Debug)]
pub struct Model {
    selected_task: TaskId,
}

#[derive(Clone)]
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
    EscapeKey,
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
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.set_description(new_description);
                });
        }
        Msg::SelectedTaskProjectChanged(new_project) => {
            let new_project = new_project.trim();
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.set_project(if new_project.is_empty() {
                        Vec::new()
                    } else {
                        new_project
                            .split('.')
                            .map(std::borrow::ToOwned::to_owned)
                            .collect()
                    });
                });
        }
        Msg::SelectedTaskTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.set_tags(if new_tags.is_empty() {
                        Vec::new()
                    } else {
                        let mut tags: Vec<_> = new_tags
                            .split_whitespace()
                            .map(|s| s.trim().to_owned())
                            .collect();
                        if new_end {
                            tags.push(String::new());
                        }
                        tags
                    });
                });
        }
        Msg::SelectedTaskPriorityChanged(new_priority) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.set_priority(match Priority::try_from(new_priority) {
                        Ok(p) => Some(p),
                        Err(()) => None,
                    });
                });
        }
        Msg::SelectedTaskNotesChanged(new_notes) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.set_notes(new_notes);
                });
        }
        Msg::SelectedTaskDueDateChanged(new_date) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
                    match new_date {
                        Ok(new_date) => {
                            let due = task.due();
                            match due {
                                None => task.set_due(new_date.and_hms_opt(0, 0, 0).map(|date| {
                                    DateTime(chrono::DateTime::from_utc(date, chrono::Utc))
                                })),

                                Some(due) => {
                                    let due = due
                                        .0
                                        .with_year(new_date.year())
                                        .and_then(|due| due.with_month(new_date.month()))
                                        .and_then(|due| due.with_day(new_date.day()))
                                        .map(DateTime);
                                    task.set_due(due);
                                }
                            }
                        }
                        Err(_) => task.set_due(None),
                    }
                });
        }
        Msg::SelectedTaskDueTimeChanged(new_time) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
                    match new_time {
                        Ok(new_time) => {
                            let due = task.due();
                            match due {
                                None => {
                                    let now = chrono::offset::Utc::now();
                                    let now = now
                                        .with_hour(new_time.hour())
                                        .and_then(|now| now.with_minute(new_time.minute()))
                                        .map(DateTime);
                                    task.set_due(now);
                                }
                                Some(due) => {
                                    let due = due
                                        .0
                                        .with_hour(new_time.hour())
                                        .and_then(|due| due.with_minute(new_time.minute()))
                                        .map(DateTime);
                                    task.set_due(due);
                                }
                            }
                        }
                        Err(_) => task.set_due(None),
                    }
                });
        }
        Msg::SelectedTaskScheduledDateChanged(new_date) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
                    match new_date {
                        Ok(new_date) => {
                            let scheduled = task.scheduled();
                            match scheduled {
                                None => task.set_scheduled(
                                    new_date
                                        .and_hms_opt(0, 0, 0)
                                        .map(|date| chrono::DateTime::from_utc(date, chrono::Utc))
                                        .map(DateTime),
                                ),
                                Some(scheduled) => {
                                    let scheduled = scheduled
                                        .0
                                        .with_year(new_date.year())
                                        .and_then(|scheduled| {
                                            scheduled.with_month(new_date.month())
                                        })
                                        .and_then(|scheduled| scheduled.with_day(new_date.day()))
                                        .map(DateTime);
                                    task.set_scheduled(scheduled);
                                }
                            }
                        }
                        Err(_) => task.set_scheduled(None),
                    }
                });
        }
        Msg::SelectedTaskScheduledTimeChanged(new_time) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
                    match new_time {
                        Ok(new_time) => {
                            let scheduled = task.scheduled();
                            match scheduled {
                                None => {
                                    let now = chrono::offset::Utc::now();
                                    let now = now
                                        .with_hour(new_time.hour())
                                        .and_then(|now| now.with_minute(new_time.minute()))
                                        .map(DateTime);
                                    task.set_scheduled(now);
                                }
                                Some(scheduled) => {
                                    let scheduled = scheduled
                                        .0
                                        .with_hour(new_time.hour())
                                        .and_then(|scheduled| {
                                            scheduled.with_minute(new_time.minute())
                                        })
                                        .map(DateTime);
                                    task.set_scheduled(scheduled);
                                }
                            }
                        }
                        Err(_) => task.set_scheduled(None),
                    }
                });
        }
        Msg::SelectedTaskRecurAmountChanged(new_amount) => {
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    if let Ok(n) = new_amount.parse::<u16>() {
                        if n > 0 {
                            task.set_recur(Some(Recur {
                                amount: n,
                                unit: task.recur().as_ref().map_or(RecurUnit::Week, |r| r.unit),
                            }));
                        } else {
                            task.set_recur(None);
                        }
                    }
                });
        }
        Msg::SelectedTaskRecurUnitChanged(new_unit) => global_model.document.change_task(
            &model.selected_task,
            |task| match RecurUnit::try_from(new_unit) {
                Ok(unit) => task.set_recur(Some(Recur {
                    amount: task.recur().as_ref().map_or(1, |r| r.amount),
                    unit,
                })),
                Err(()) => task.set_recur(None),
            },
        ),
        Msg::DeleteSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            if let Some(task) = global_model.document.get_task(&model.selected_task) {
                match task.status() {
                    Status::Pending | Status::Completed | Status::Waiting | Status::Recurring => {
                        global_model
                            .document
                            .change_task(&model.selected_task, |task| {
                                task.delete();
                            });
                    }
                    Status::Deleted => match window().confirm_with_message(
                        "Are you sure you want to permanently delete this task?",
                    ) {
                        Ok(true) => {
                            /* already removed from set so just don't add it back */
                            global_model.document.remove_task(&model.selected_task);
                        }
                        Ok(false) | Err(_) => {}
                    },
                }
            }
        }
        Msg::CompleteSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.complete();
                });
        }
        Msg::StartSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.activate();
                });
        }
        Msg::StopSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.deactivate();
                });
        }
        Msg::MoveSelectedTaskToPending => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            global_model
                .document
                .change_task(&model.selected_task, |task| {
                    task.restore();
                });
        }
        Msg::EscapeKey => {
            orders.request_url(Urls::new(&global_model.base_url).home());
        }
    }
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let task = global_model
        .document
        .get_task(&model.selected_task)
        .expect("the given task to exist");
    div![view_selected_task(task, &global_model.document)]
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn view_selected_task(task: &Task, document: &Document) -> Node<GMsg> {
    let is_pending = matches!(task.status(), Status::Pending);
    let start = task.start();
    let end = task.end();
    let urgency = urgency::calculate(task);
    let active = task.start().is_some();
    let is_next = task.tags().contains(&"next".to_owned());
    let project_lowercase = task.project().join(".").to_lowercase();
    let project_suggestions = document
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
    let mut tags_suggestions = document
        .tasks()
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
                        Some(saved_tag.clone())
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
        urgency.map_or_else(
            || empty![],
            |urgency| {
                div![
                    C!["pl-2"],
                    span![C!["font-bold"], "Urgency: "],
                    plain![format!("{urgency:.2}")]
                ]
            }
        ),
        div![
            C!["pl-2"],
            span![C!["font-bold"], "Entry: "],
            task.entry().0.to_string()
        ],
        start.as_ref().map_or_else(
            || empty![],
            |start| {
                div![
                    C!["pl-2"],
                    span![C!["font-bold"], "Start: "],
                    start.0.to_string()
                ]
            }
        ),
        end.as_ref().map_or_else(
            || empty![],
            |end| {
                div![
                    C!["pl-2"],
                    span![C!["font-bold"], "End: "],
                    end.0.to_string()
                ]
            }
        ),
        view_text_input(
            "Description",
            task.description(),
            true,
            BTreeSet::new(),
            |s| GMsg::ViewTask(Msg::SelectedTaskDescriptionChanged(s))
        ),
        view_text_input(
            "Project",
            &task.project().join("."),
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
                        At::Value => task.tags().join(" "),
                        At::AutoFocus => AtValue::Ignored
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(Msg::SelectedTaskTagsChanged(
                        s
                    )))
                ],
                if task.tags().join(" ").is_empty() {
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
                        let tags = task.tags().join(" ");
                        button![
                            C!["mr-2", "mt-2", "px-1", "bg-gray-200"],
                            mouse_ev(Ev::Click, move |_| {
                                if tags.ends_with(' ') || tags.is_empty() {
                                    GMsg::ViewTask(Msg::SelectedTaskTagsChanged(format!(
                                        "{tags} {sug_clone}"
                                    )))
                                } else {
                                    let split_tags = tags.split_whitespace().collect::<Vec<_>>();
                                    let tags = split_tags
                                        .iter()
                                        .take(split_tags.len() - 1)
                                        .map(std::borrow::ToOwned::to_owned)
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    GMsg::ViewTask(Msg::SelectedTaskTagsChanged(format!(
                                        "{tags} {sug_clone}"
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
                            At::Selected => if matches!(task.priority(), Some(Priority::Low)) {
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
                            At::Selected => if matches!(task.priority(), Some(Priority::Medium)) {
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
                            At::Selected => if matches!(task.priority(), Some(Priority::High)) {
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
                        At::Value => task.due().as_ref().map_or_else(String::new, |due| due.0.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.due().as_ref().map_or_else(String::new, |due| due.0.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueTimeChanged(s)
                    ))
                ],
                task.due().as_ref().map_or_else(
                    || empty![],
                    |due| {
                        span![
                            C!["ml-2"],
                            duration_string(
                                due.0.signed_duration_since(chrono::offset::Utc::now())
                            )
                        ]
                    }
                )
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
                        At::Value => task.scheduled().as_ref().map_or_else(String::new, |scheduled| scheduled.0.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.scheduled().as_ref().map_or_else(String::new, |scheduled| scheduled.0.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledTimeChanged(s)
                    ))
                ],
                task.scheduled().as_ref().map_or_else(
                    || empty![],
                    |scheduled| {
                        span![
                            C!["ml-2"],
                            duration_string(
                                scheduled
                                    .0
                                    .signed_duration_since(chrono::offset::Utc::now())
                            )
                        ]
                    }
                )
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
                        At::Value => task.notes(),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskNotesChanged(s)
                    ))
                ],
                if task.notes().is_empty() {
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
                        view_button_str("Stop", GMsg::ViewTask(Msg::StopSelectedTask))
                    } else {
                        view_button_str("Start", GMsg::ViewTask(Msg::StartSelectedTask))
                    }
                ]
            ),
            IF!(is_pending =>
                div![ view_button_str("Complete", GMsg::ViewTask(Msg::CompleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Pending|Status::Waiting|Status::Recurring) =>
                div![ view_button_str("Delete", GMsg::ViewTask(Msg::DeleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button_str("Permanently delete", GMsg::ViewTask(Msg::DeleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button_str("Undelete", GMsg::ViewTask(Msg::MoveSelectedTaskToPending))]
            ),
            IF!(matches!(task.status(), Status::Completed) =>
                div![ view_button_str("Uncomplete", GMsg::ViewTask(Msg::MoveSelectedTaskToPending))]
            ),
            view_button_str("Close", GMsg::SelectTask(None))
        ]
    ]
}
