#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{auth::Provider, GlobalModel, Msg as GMsg};

pub fn init() -> Model {
    let auth_provider = Provider::load_from_session();
    Model { auth_provider }
}

#[derive(Debug)]
pub struct Model {
    auth_provider: Option<Provider>,
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
                Provider::Google.logo(),
                "Sign in with Google",
            ]
        ),
        IF!(
            model.auth_provider.is_some() =>
            a![
                C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                attrs! {At::Href => "/auth/google/sign_out"},
                model.auth_provider.as_ref().unwrap().logo(),
                "Sign out with Google",
            ]
        ),
    ]
}
