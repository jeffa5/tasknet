use std::{
    collections::{BTreeSet, HashMap},
    convert::TryFrom,
};

use automerge::Path;
use chrono::{Datelike, Timelike};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_text_input},
    task::{Priority, RecurUnit, Status, Task},
    urgency, GlobalModel, Msg as GMsg, Recur, Urls,
};

const ESCAPE_KEY: &str = "Escape";

pub fn init(uuid: uuid::Uuid, orders: &mut impl Orders<GMsg>) -> Model {
    orders.stream(streams::window_event(Ev::KeyUp, |event| {
        let key_event: web_sys::KeyboardEvent = event.unchecked_into();
        match key_event.key().as_ref() {
            ESCAPE_KEY => Some(GMsg::ViewTask(Msg::EscapeKey)),
            _ => None,
        }
    }));
    Model {
        selected_task: uuid,
    }
}

#[derive(Debug)]
pub struct Model {
    selected_task: uuid::Uuid,
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
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, _task| {
                    Task::set_description(path, &new_description)
                });
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::SelectedTaskProjectChanged(new_project) => {
            let new_project = new_project.trim();
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, _task| {
                    Task::set_project(
                        path,
                        if new_project.is_empty() {
                            Vec::new()
                        } else {
                            new_project
                                .split('.')
                                .map(std::borrow::ToOwned::to_owned)
                                .collect()
                        },
                    )
                });
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::SelectedTaskTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, _task| {
                    Task::set_tags(
                        path,
                        if new_tags.is_empty() {
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
                        },
                    )
                });
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::SelectedTaskPriorityChanged(new_priority) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     task.set_priority(match Priority::try_from(new_priority) {
            //         Ok(p) => Some(p),
            //         Err(()) => None,
            //     });
            // }
        }
        Msg::SelectedTaskNotesChanged(new_notes) => {
            // if let some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     task.set_notes(new_notes)
            // }
        }
        Msg::SelectedTaskDueDateChanged(new_date) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
            //     match new_date {
            //         Ok(new_date) => {
            //             let due = task.due();
            //             match due {
            //                 None => task.set_due(Some(chrono::DateTime::from_utc(
            //                     new_date.and_hms(0, 0, 0),
            //                     chrono::Utc,
            //                 ))),
            //                 Some(due) => {
            //                     let due = due
            //                         .with_year(new_date.year())
            //                         .and_then(|due| due.with_month(new_date.month()))
            //                         .and_then(|due| due.with_day(new_date.day()));
            //                     task.set_due(due)
            //                 }
            //             }
            //         }
            //         Err(_) => task.set_due(None),
            //     }
            // }
        }
        Msg::SelectedTaskDueTimeChanged(new_time) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
            //     match new_time {
            //         Ok(new_time) => {
            //             let due = task.due();
            //             match due {
            //                 None => {
            //                     let now = chrono::offset::Utc::now();
            //                     let now = now
            //                         .with_hour(new_time.hour())
            //                         .and_then(|now| now.with_minute(new_time.minute()));
            //                     task.set_due(now)
            //                 }
            //                 Some(due) => {
            //                     let due = due
            //                         .with_hour(new_time.hour())
            //                         .and_then(|due| due.with_minute(new_time.minute()));
            //                     task.set_due(due)
            //                 }
            //             }
            //         }
            //         Err(_) => task.set_due(None),
            //     }
            // }
        }
        Msg::SelectedTaskScheduledDateChanged(new_date) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     let new_date = chrono::NaiveDate::parse_from_str(&new_date, "%Y-%m-%d");
            //     match new_date {
            //         Ok(new_date) => {
            //             let scheduled = task.scheduled();
            //             match scheduled {
            //                 None => task.set_scheduled(Some(chrono::DateTime::from_utc(
            //                     new_date.and_hms(0, 0, 0),
            //                     chrono::Utc,
            //                 ))),
            //                 Some(scheduled) => {
            //                     let scheduled = scheduled
            //                         .with_year(new_date.year())
            //                         .and_then(|scheduled| scheduled.with_month(new_date.month()))
            //                         .and_then(|scheduled| scheduled.with_day(new_date.day()));
            //                     task.set_scheduled(scheduled)
            //                 }
            //             }
            //         }
            //         Err(_) => task.set_scheduled(None),
            //     }
            // }
        }
        Msg::SelectedTaskScheduledTimeChanged(new_time) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     let new_time = chrono::NaiveTime::parse_from_str(&new_time, "%H:%M");
            //     match new_time {
            //         Ok(new_time) => {
            //             let scheduled = task.scheduled();
            //             match scheduled {
            //                 None => {
            //                     let now = chrono::offset::Utc::now();
            //                     let now = now
            //                         .with_hour(new_time.hour())
            //                         .and_then(|now| now.with_minute(new_time.minute()));
            //                     task.set_scheduled(now)
            //                 }
            //                 Some(scheduled) => {
            //                     let scheduled = scheduled
            //                         .with_hour(new_time.hour())
            //                         .and_then(|scheduled| scheduled.with_minute(new_time.minute()));
            //                     task.set_scheduled(scheduled)
            //                 }
            //             }
            //         }
            //         Err(_) => task.set_scheduled(None),
            //     }
            // }
        }
        Msg::SelectedTaskRecurAmountChanged(new_amount) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     if let Ok(n) = new_amount.parse::<u16>() {
            //         if n > 0 {
            //             task.set_recur(Some(Recur {
            //                 amount: n,
            //                 unit: task.recur().as_ref().map_or(RecurUnit::Week, |r| r.unit),
            //             }))
            //         } else {
            //             task.set_recur(None)
            //         }
            //     }
            // }
        }
        Msg::SelectedTaskRecurUnitChanged(new_unit) => {
            // if let Some(task) = &mut global_model.document.get_mut(&model.selected_task) {
            //     match RecurUnit::try_from(new_unit) {
            //         Ok(unit) => task.set_recur(Some(Recur {
            //             amount: task.recur().as_ref().map_or(1, |r| r.amount),
            //             unit,
            //         })),
            //         Err(()) => task.set_recur(None),
            //     }
            // }
        }
        Msg::DeleteSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            // if let Some(task) = global_model.document.get_mut(&model.selected_task) {
            //     match task.status() {
            //         Status::Pending | Status::Completed | Status::Waiting | Status::Recurring => {
            //             task.delete();
            //         }
            //         Status::Deleted => match window().confirm_with_message(
            //             "Are you sure you want to permanently delete this task?",
            //         ) {
            //             Ok(true) => {
            //                 /* already removed from set so just don't add it back */
            //                 global_model.document.remove(&model.selected_task);
            //             }
            //             Ok(false) | Err(_) => {}
            //         },
            //     }
            // }
        }
        Msg::CompleteSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, _task| Task::complete(path));
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::StartSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, task| task.activate(path));
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::StopSelectedTask => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, _task| Task::deactivate(path));
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::MoveSelectedTaskToPending => {
            orders.request_url(Urls::new(&global_model.base_url).home());
            let msg = global_model
                .document
                .change_task(&model.selected_task, |path, task| task.restore(path));
            if let Some(msg) = msg {
                orders.send_msg(msg);
            }
        }
        Msg::EscapeKey => {
            orders.request_url(Urls::new(&global_model.base_url).home());
        }
    }
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let task = global_model
        .document
        .task(&model.selected_task)
        .expect("the given task to exist");
    div![view_selected_task(&task, &global_model.document.tasks()),]
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn view_selected_task(task: &Task, tasks: &HashMap<uuid::Uuid, Task>) -> Node<GMsg> {
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
                        At::Value => task.due().map_or_else(String::new, |due| due.format("%Y-%m-%d").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.due().map_or_else(String::new, |due| due.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskDueTimeChanged(s)
                    ))
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
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledDateChanged(s)
                    ))
                ],
                input![
                    attrs! {
                        At::Type => "time",
                        At::Value => task.scheduled().map_or_else(String::new, |scheduled| scheduled.format("%H:%M").to_string()),
                    },
                    input_ev(Ev::Input, |s| GMsg::ViewTask(
                        Msg::SelectedTaskScheduledTimeChanged(s)
                    ))
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
                        view_button("Stop", GMsg::ViewTask(Msg::StopSelectedTask))
                    } else {
                        view_button("Start", GMsg::ViewTask(Msg::StartSelectedTask))
                    }
                ]
            ),
            IF!(is_pending =>
                div![ view_button("Complete", GMsg::ViewTask(Msg::CompleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Pending|Status::Waiting|Status::Recurring) =>
                div![ view_button("Delete", GMsg::ViewTask(Msg::DeleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button("Permanently delete", GMsg::ViewTask(Msg::DeleteSelectedTask))]
            ),
            IF!(matches!(task.status(), Status::Deleted) =>
                div![ view_button("Undelete", GMsg::ViewTask(Msg::MoveSelectedTaskToPending))]
            ),
            IF!(matches!(task.status(), Status::Completed) =>
                div![ view_button("Uncomplete", GMsg::ViewTask(Msg::MoveSelectedTaskToPending))]
            ),
            view_button("Close", GMsg::SelectTask(None))
        ]
    ]
}
