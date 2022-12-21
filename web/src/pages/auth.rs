#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{GlobalModel, Msg as GMsg};

pub fn init() -> Model {
    let auth_provider = crate::auth::provider();
    Model { auth_provider }
}

#[derive(Debug)]
pub struct Model {
    auth_provider: Option<String>,
}

#[derive(Clone)]
pub enum Msg {}

pub fn update(
    _msg: Msg,
    _global_model: &mut GlobalModel,
    _model: &mut Model,
    _orders: &mut impl Orders<GMsg>,
) {
}

pub fn view(_global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let google_logo = img![
        C!["inline", "pr-2"],
        attrs! {At::Src => "/assets/btn_google_light_normal_ios.svg"}
    ];
    div![
        C![
            "flex",
            "flex-col",
            "mx-auto",
            "bg-gray-100",
            "p-2",
            "border-4",
            "border-gray-200"
        ],
        IF!(
            model.auth_provider.is_none() =>
            a![
                C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                attrs! {At::Href => "/auth/google/sign_in"},
                google_logo.clone(),
                "Sign in with Google",
            ]
        ),
        IF!(
            model.auth_provider.is_some() =>
            a![
                C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                attrs! {At::Href => "/auth/google/sign_out"},
                google_logo,
                "Sign out with Google",
            ]
        ),
    ]
}
