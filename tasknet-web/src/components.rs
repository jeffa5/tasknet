use std::collections::{BTreeMap, BTreeSet};

use kratos_api::models::{UiContainer, UiNodeAttributes};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::Msg;

pub fn view_button(text: &str, msg: Msg, disabled: bool) -> Node<Msg> {
    if disabled {
        button![
            attrs! {
                At::Disabled => AtValue::None,
            },
            C![
                "bg-gray-200",
                "py-2",
                "px-4",
                "m-2",
                "hover:bg-gray-300",
                "opacity-60",
            ],
            mouse_ev(Ev::Click, |_| msg),
            text
        ]
    } else {
        button![
            C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300",],
            mouse_ev(Ev::Click, |_| msg),
            text
        ]
    }
}

pub fn view_link(text: &str, url: &str, disabled: bool) -> Node<Msg> {
    if disabled {
        a![
            attrs! {
                At::Disabled => AtValue::None,
                At::Href => AtValue::Some(url.to_owned()),
            },
            C![
                "bg-gray-200",
                "py-2",
                "px-4",
                "m-2",
                "hover:bg-gray-300",
                "opacity-60",
            ],
            text
        ]
    } else {
        a![
            attrs! {
                At::Href => AtValue::Some(url.to_owned()),
            },
            C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300",],
            text
        ]
    }
}

pub fn view_checkbox(name: &str, title: &str, checked: bool, msg: Msg) -> Node<Msg> {
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

pub fn view_text_input(
    name: &str,
    value: &str,
    default: &str,
    autofocus: bool,
    suggestions: BTreeSet<String>,
    f: impl FnOnce(String) -> Msg + Clone + 'static,
    valid: bool,
) -> Node<Msg> {
    let also_f = f.clone();
    let also_also_f = f.clone();
    div![
        C!["flex", "flex-col", "px-2", "mb-2"],
        div![C!["font-bold"], name],
        div![
            C!["flex", "flex-row"],
            input![
                C![
                    "flex-grow",
                    "border",
                    if valid { "" } else { "border-red-600" },
                    "mr-2"
                ],
                attrs! {
                    At::Value => value,
                    At::AutoFocus => if autofocus { AtValue::None } else { AtValue::Ignored }
                },
                input_ev(Ev::Input, f)
            ],
            if value == default {
                pre![" "]
            } else {
                let string_default = default.to_owned();
                button![
                    mouse_ev(Ev::Click, |_| also_f(string_default)),
                    div![C!["text-red-600"], "X"]
                ]
            }
        ],
        div![
            C!["flex", "flex-row", "overflow-hidden"],
            suggestions
                .into_iter()
                .map(|sug| {
                    let sug_clone = sug.clone();
                    let new_f = also_also_f.clone();
                    button![
                        C!["mr-2", "mt-2", "px-1", "bg-gray-200"],
                        mouse_ev(Ev::Click, |_| new_f(sug_clone)),
                        sug
                    ]
                })
                .collect::<Vec<_>>()
        ]
    ]
}

pub fn view_number_input_tr(
    name: &str,
    value: i64,
    default: i64,
    f: impl FnOnce(i64) -> Msg + Clone + 'static,
) -> Node<Msg> {
    let also_f = f.clone();
    tr![
        td![span![C!["font-bold", "mr-2"], name]],
        td![input![
            C!["mr-2"],
            attrs! {
                At::Type => "number",
                At::Value => value,
            },
            input_ev(Ev::Change, |s| f(s.parse().unwrap()))
        ]],
        td![if value == default {
            pre![" "]
        } else {
            button![
                mouse_ev(Ev::Click, move |_| also_f(default)),
                span![C!["text-red-600"], "X"]
            ]
        }],
    ]
}

pub fn duration_string(duration: chrono::Duration) -> String {
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
    } else if duration.num_seconds() > -1 {
        "now".to_owned()
    } else if duration.num_seconds() > -60 {
        format!("{}s", duration.num_seconds())
    } else if duration.num_minutes() > -60 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_hours() > -24 {
        format!("{}h", duration.num_hours())
    } else if duration.num_days() > -7 {
        format!("{}d", duration.num_days())
    } else {
        format!("{}w", duration.num_weeks())
    }
}

pub fn view_ui(ui: &UiContainer) -> Node<Msg> {
    let mut nodes: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for node in ui.nodes.iter() {
        nodes.entry(node.group.clone()).or_default().push(node);
    }
    div![
        C!["flex", "flex-col"],
        form![
            attrs! {
                At::Action =>ui.action,
                At::Method => ui.method,
            },
            nodes.iter().map(|(group, nodes)| {
                let all_hidden = nodes.iter().all(|node| match &*node.attributes {
                    UiNodeAttributes::UiNodeInputAttributes {
                        _type, ..} => {
                            _type == "hidden"
                        }
                    _ => false
                });
                let inner = div![
                    if group != "default" {
                        legend![C!["font-bold"], group]
                    } else {
                        empty!()
                    },
                    nodes
                    .iter()
                    .map(|node| match &*node.attributes {
                        UiNodeAttributes::UiNodeInputAttributes {
                            disabled,
                            label: _,
                            name,
                            onclick: _,
                            pattern,
                            required,
                            _type,
                            value,
                        } => {
                            if _type  == "submit" {
                                button![
                                    C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300",],
                                    attrs! {
                                        At::Name => name,
                                        At::Type => _type,
                                        At::Value => if let Some(value) = value.as_ref() {
                                            AtValue::Some(value.to_string().replace("\"",""))
                                        } else { AtValue::Ignored },
                                        At::Disabled => if *disabled {AtValue::None} else {AtValue::Ignored},
                                    },
                                    if let Some(label) = &node.meta.label {
                                        span![&label.text]
                                    } else {
                                        empty!()
                                    },
                                ]
                            } else {
                                label![
                                    if let Some(label) = &node.meta.label {
                                        span![C!["mr-2"], &label.text, ":"]
                                    } else {
                                        empty!()
                                    },
                                    br![],
                                    input![
                                        C![
                                            "flex-grow",
                                            "border",
                                            "mr-2",
                                        ],
                                        attrs! {
                                            At::Name => name,
                                            At::Type => _type,
                                            At::Value => if let Some(value) = value.as_ref() {
                                                AtValue::Some(value.to_string().replace("\"",""))
                                            } else { AtValue::Ignored },
                                            At::Disabled => if *disabled {AtValue::None} else {AtValue::Ignored},
                                        }
                                    ],
                                    br![],
                                ]
                            }
                        }
                        _ => {
                            div!["non input"]
                        }
                    })
                ];
                if all_hidden {
                    inner
                } else {
                    fieldset![inner]
                }
            })
        ],
        if let Some(messages) = &ui.messages {
            messages.iter().map(|text| {
                div![
                    &text.text
                ]
            }).collect::<Vec<_>>()
        } else {
            vec![]
        }
    ]
}
