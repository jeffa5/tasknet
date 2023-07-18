#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{auth::Provider, GlobalModel, Msg as GMsg};
use gloo_console::log;
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
        public_doc_id: String::new(),
    }
}

#[derive(Debug)]
pub struct Model {
    auth_provider: Option<Provider>,
    providers: Option<Providers>,
    public_doc_id: String,
}

#[derive(Clone)]
pub enum Msg {
    FetchedProviders(Providers),
    PublicDocIdChanged(String),
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
        Msg::PublicDocIdChanged(new_id) => {
            model.public_doc_id = new_id;
        }
    }
}

pub fn view(_global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    let public_provider = form![
        C!["py-1", "px-2", "m-1"],
        Provider::Public.logo(),
        "Access a public document",
        br!(),
        label!["Document ID"],
        input![
            attrs! {
                At::Value => model.public_doc_id
            },
            input_ev(Ev::Input, |s| GMsg::Auth(Msg::PublicDocIdChanged(s)))
        ],
        a![
            C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
            attrs! {At::Href => format!("/auth/public/sign_in?doc_id={}", model.public_doc_id) },
            "Use",
        ],
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
        if let Some(providers) = &model.providers {
            if model.auth_provider.is_none() {
                log!(format!("providers: {:?}", providers));
                div![
                    IF!(providers.google.enabled => a![
                        C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                        attrs! {At::Href => "/auth/google/sign_in"},
                        Provider::Google.logo(),
                        "Sign in with Google",
                    ]),
                    IF!(providers.public.enabled => public_provider)
                ]
            } else {
                let provider = model.auth_provider.as_ref().unwrap();
                match provider {
                    Provider::Public => div![a![
                        C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                        attrs! {At::Href => "/auth/public/sign_out"},
                        provider.logo(),
                        "Sign out from Public",
                    ]],
                    Provider::Google => {
                        div![a![
                            C!["bg-gray-200", "py-1", "px-2", "m-1", "hover:bg-gray-300"],
                            attrs! {At::Href => "/auth/google/sign_out"},
                            provider.logo(),
                            "Sign out with Google",
                        ],]
                    }
                }
            }
        } else {
            div![
                div![C!["py-1", "px-2", "m-1"], "No auth providers available"],
                public_provider
            ]
        }
    ]
}
