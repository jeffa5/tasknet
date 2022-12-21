#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{GlobalModel, Msg as GMsg};

pub const SESSION_COOKIE: &str = "session";
pub const AUTH_PROVIDER_COOKIE: &str = "auth-provider";

pub fn init() -> Model {
    let have_session = cookies()
        .map(|cookie_jar| cookie_jar.get(SESSION_COOKIE).is_some())
        .unwrap_or_default();

    let auth_provider = if have_session {
        cookies().and_then(|cookie_jar| {
            cookie_jar
                .get(AUTH_PROVIDER_COOKIE)
                .map(|cookie| cookie.value())
                .map(std::borrow::ToOwned::to_owned)
        })
    } else {
        None
    };

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
