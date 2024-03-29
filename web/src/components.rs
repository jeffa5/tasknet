use std::collections::BTreeSet;

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::Msg;

#[derive(Debug, Default)]
pub struct ButtonOptions {
    pub disabled: bool,
}

pub fn view_button_str(text: &str, msg: Msg) -> Node<Msg> {
    button![
        C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300"],
        mouse_ev(Ev::Click, |_| msg),
        text
    ]
}

pub fn view_button(node: Node<Msg>, msg: Msg, options: &ButtonOptions) -> Node<Msg> {
    let bg = if options.disabled {
        "bg-gray-100"
    } else {
        "bg-gray-200"
    };
    let text_colour = if options.disabled {
        "text-gray-600"
    } else {
        "text-current"
    };
    let hover = if options.disabled {
        ""
    } else {
        "hover:bg-gray-300"
    };
    button![
        C![bg, text_colour, "py-2", "px-4", "m-2", hover],
        attrs! {
            At::Disabled => if options.disabled { AtValue::None } else { AtValue::Ignored },
        },
        mouse_ev(Ev::Click, |_| msg),
        node
    ]
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
    autofocus: bool,
    suggestions: BTreeSet<String>,
    f: impl FnOnce(String) -> Msg + Clone + 'static,
) -> Node<Msg> {
    let also_f = f.clone();
    let also_also_f = f.clone();
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
