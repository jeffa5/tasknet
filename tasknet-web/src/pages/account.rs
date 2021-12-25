use kratos_api::models::{SelfServiceSettingsFlow, UiContainer};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};
use web_sys::UrlSearchParams;

use crate::{components::view_ui, GlobalModel, Msg as GMsg};

pub fn init(_global_model: &GlobalModel, orders: &mut impl Orders<GMsg>) -> Model {
    let model = Model { ui: None };
    orders.send_msg(GMsg::Account(Msg::GetSettings));
    model
}

#[derive(Debug)]
pub struct Model {
    ui: Option<UiContainer>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    GetSettings,
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
        Msg::GetSettings => {
            orders.perform_cmd(async move {
                if let Some(flow) =
                    UrlSearchParams::new_with_str(&window().location().search().unwrap())
                        .unwrap()
                        .get("flow")
                {
                    let response = fetch(
                        Request::new(format!("/kratos/self-service/settings/flows?id={}", flow))
                            .header(Header::custom("Accept", "application/json")),
                    )
                    .await
                    .expect("HTTP request failed");
                    let value = response.json::<SelfServiceSettingsFlow>().await.unwrap();
                    log!(value);

                    Some(GMsg::Account(Msg::SetUi(*value.ui)))
                } else {
                    None
                }
            });
        }
        Msg::SetUi(ui) => model.ui = Some(ui),
    }
}

pub fn view(_global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div![
        C!["flex", "flex-col"],
        h1![C!["text-lg", "font-bold"], "Account"],
        div![
            C!["flex", "flex-col"],
            if let Some(ui) = model.ui.as_ref() {
                view_ui(ui)
            } else {
                div!["no ui response yet"]
            }
        ],
    ]
}
