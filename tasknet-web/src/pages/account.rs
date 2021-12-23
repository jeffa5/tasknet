use kratos_api::models::{UiContainer, UiNode, UiNodeAttributes};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{view_button, view_text_input},
    GlobalModel, Msg as GMsg,
};

pub fn init(_global_model: &GlobalModel, orders: &mut impl Orders<GMsg>) -> Model {
    let model = Model {
        username: String::new(),
        password: String::new(),
        is_logged_in: false,
        ui: None,
    };
    if !model.is_logged_in {
        orders.send_msg(GMsg::Account(Msg::Register));
    }
    model
}

#[derive(Debug)]
pub struct Model {
    username: String,
    password: String,
    is_logged_in: bool,
    ui: Option<UiContainer>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    SetUsername(String),
    SetPassword(String),
    Login,
    Register,
    SetUi(UiContainer),
}

#[allow(clippy::too_many_lines)]
pub fn update(
    msg: Msg,
    _global_model: &mut GlobalModel,
    model: &mut Model,
    orders: &mut impl Orders<GMsg>,
) {
    match msg {
        Msg::SetUsername(username) => {
            model.username = username;
        }
        Msg::SetPassword(password) => {
            model.password = password;
        }
        Msg::Login => {
            orders.perform_cmd(async {
                let response = fetch(
                    Request::new("/kratos/self-service/login/browser")
                        .header(Header::custom("Accept", "application/json")),
                )
                .await
                .expect("HTTP request failed");

                let value = response
                    .check_status() // ensure we've got 2xx status
                    .expect("status check failed")
                    .json::<kratos_api::models::SelfServiceLoginFlow>()
                    .await
                    .expect("deserialization failed");

                log!(value);
            });
        }
        Msg::Register => {
            orders.perform_cmd(async {
                let response = fetch(
                    Request::new("/kratos/self-service/registration/browser")
                        .header(Header::custom("Accept", "application/json")),
                )
                .await
                .expect("HTTP request failed");

                let value = response
                    .check_status() // ensure we've got 2xx status
                    .expect("status check failed")
                    .json::<kratos_api::models::SelfServiceRegistrationFlow>()
                    .await
                    .expect("deserialization failed");

                log!(value);
                GMsg::Account(Msg::SetUi(*value.ui))
            });
        }
        Msg::SetUi(ui) => {
            model.ui = Some(ui);
        }
    }
}

pub fn view(_global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![
        C!["flex", "flex-col"],
        h1![C!["text-lg", "font-bold"], "Account"],
        div![
            C!["flex", "flex-col"],
            view_text_input(
                "Username",
                &model.username,
                "",
                false,
                Default::default(),
                |s| GMsg::Account(Msg::SetUsername(s)),
                true,
            ),
            view_text_input(
                "Password",
                &model.password,
                "",
                false,
                Default::default(),
                |s| GMsg::Account(Msg::SetPassword(s)),
                true,
            ),
            if model.is_logged_in {
                view_button(
                    "Login",
                    GMsg::Account(Msg::Login),
                    model.username.is_empty() || model.password.is_empty(), // can't login without one of these fields
                )
            } else {
                view_button("Register", GMsg::Account(Msg::Register), false)
            },
            if let Some(ui) = model.ui.as_ref() {
                view_ui(ui)
            } else {
                div!["no ui response yet"]
            }
        ],
    ]
}

fn get_provider(node: &UiNode) -> String {
    match &*node.attributes {
        kratos_api::models::UiNodeAttributes::UiNodeAnchorAttributes { href, id, title } => todo!(),
        kratos_api::models::UiNodeAttributes::UiNodeImageAttributes {
            height,
            id,
            src,
            width,
        } => todo!(),
        kratos_api::models::UiNodeAttributes::UiNodeInputAttributes {
            disabled,
            label,
            name,
            onclick,
            pattern,
            required,
            _type,
            value,
        } => match value.as_ref().unwrap() {
            serde_json::Value::Null => todo!(),
            serde_json::Value::Bool(_) => todo!(),
            serde_json::Value::Number(_) => todo!(),
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(_) => todo!(),
            serde_json::Value::Object(_) => todo!(),
        },
        kratos_api::models::UiNodeAttributes::UiNodeScriptAttributes {
            _async,
            crossorigin,
            id,
            integrity,
            referrerpolicy,
            src,
            _type,
        } => todo!(),
        kratos_api::models::UiNodeAttributes::UiNodeTextAttributes { id, text } => todo!(),
    }
}

fn view_ui(ui: &UiContainer) -> Node<GMsg> {
    div![form![
        attrs! {
            At::Action =>ui.action,
            At::Method => ui.method,
        },
        ui.nodes.iter().map(|node| match &*node.attributes {
            UiNodeAttributes::UiNodeInputAttributes {
                disabled,
                label,
                name,
                onclick,
                pattern,
                required,
                _type,
                value,
            } => {
                label![
                    input![attrs! {
                        At::Name => name,
                        At::Type => _type,
                        At::Value => if let Some(value) = value.as_ref() {
                            AtValue::Some(value.to_string().replace("\"",""))
                        } else { AtValue::Ignored },
                        At::Disabled => if *disabled {AtValue::None} else {AtValue::Ignored},
                    }],
                    if let Some(label) = &node.meta.label {
                        span![&label.text]
                    } else {
                        empty!()
                    }
                ]
            }
            _ => {
                div!["non input"]
            }
        })
    ]]
}
