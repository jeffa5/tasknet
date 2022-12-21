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
    div![
        C!["flex", "flex-col", "mx-auto"],
        h2!["Google"],
        IF!(
            model.auth_provider.is_none() =>
            a![attrs! {At::Href => "/auth/google/sign_in"}, "Sign in"]
        ),
        IF!(
            model.auth_provider.is_some() =>
            a![attrs! {At::Href => "/auth/google/sign_out"}, "Sign out"]
        ),
    ]
}
