#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{auth::Provider, GlobalModel, Msg as GMsg};
use gloo_net::http::Request;
use sync::providers::Providers;

pub fn init(orders: &mut impl Orders<GMsg>) -> Model {
    let auth_provider = Provider::load_from_session();

    orders.perform_cmd(async move {
        let providers_request = Request::get("/auth/providers");
        let res = providers_request.send().await;
        if let Ok(res) = res {
            if let Ok(providers) = res.json::<Providers>().await {
                return Some(GMsg::Auth(Msg::FetchedProviders(providers)));
            }
        }
        None
    });
    Model {
        auth_provider,
        providers: None,
    }
}

#[derive(Debug)]
pub struct Model {
    auth_provider: Option<Provider>,
    providers: Option<Providers>,
}

#[derive(Clone)]
pub enum Msg {
    FetchedProviders(Providers),
}

pub fn update(
    msg: Msg,
    _global_model: &mut GlobalModel,
    model: &mut Model,
    _orders: &mut impl Orders<GMsg>,
) {
    match msg {
        Msg::FetchedProviders(providers) => {
            model.providers = Some(providers);
        }
    }
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
        if model.providers.is_none() {
            div!["No auth providers available"]
        } else {
            if model.auth_provider.is_none() {
                a![
                    C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                    attrs! {At::Href => "/auth/google/sign_in"},
                    Provider::Google.logo(),
                    "Sign in with Google",
                ]
            } else {
                a![
                    C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                    attrs! {At::Href => "/auth/google/sign_out"},
                    model.auth_provider.as_ref().unwrap().logo(),
                    "Sign out with Google",
                ]
            }
        }
    ]
}
