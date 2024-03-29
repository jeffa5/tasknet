#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::convert::TryFrom;
use wasm_sockets::{self, ConnectionStatus, EventClient};
use web_sys::CloseEvent;

use auth::Provider;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use gloo_console::error;
use gloo_console::log;

mod auth;
mod components;
mod document;
mod filters;
mod pages;
mod task;
mod urgency;

use components::{view_button, view_button_str, ButtonOptions};
use document::Document;
use filters::Filters;
use task::TaskId;
use tasknet_shared::sync::SyncMessage;

const VIEW_TASK: &str = "view";
const AUTH: &str = "auth";
const SETTINGS: &str = "settings";

fn ws_url() -> String {
    let location = window().location();
    let protocol = match location.protocol().unwrap_or_default().as_str() {
        "https:" => "wss",
        _ => "ws",
    };
    format!(
        "{}://{}{}sync",
        protocol,
        location.host().unwrap(),
        location.pathname().unwrap(),
    )
}

fn create_websocket(orders: &impl Orders<Msg>) -> EventClient {
    let msg_sender = orders.msg_sender();

    let mut client = EventClient::new(&ws_url()).expect("Failed to create websocket client");

    let send = msg_sender.clone();
    client.set_on_error(Some(Box::new(move |error| {
        error!("WS: {:#?}", error);
        send(Some(Msg::WebSocketFailed));
    })));

    let send = msg_sender.clone();
    client.set_on_connection(Some(Box::new(move |client: &EventClient| {
        log!(format!("{:#?}", client.status));
        let msg = match *client.status.borrow() {
            ConnectionStatus::Connecting => {
                log!("Connecting...");
                None
            }
            ConnectionStatus::Connected => Some(Msg::WebSocketOpened),
            ConnectionStatus::Error => Some(Msg::WebSocketFailed),
            ConnectionStatus::Disconnected => {
                log!("Disconnected");
                None
            }
        };
        send(msg);
    })));

    let send = msg_sender.clone();
    client.set_on_close(Some(Box::new(move |ev| {
        log!("WS: Connection closed");
        send(Some(Msg::WebSocketClosed(ev)));
    })));

    let send = msg_sender.clone();
    client.set_on_message(Some(Box::new(
        move |_: &EventClient, msg: wasm_sockets::Message| match msg {
            wasm_sockets::Message::Text(s) => {
                error!("received text message from websocket: {:?}", s);
            }
            wasm_sockets::Message::Binary(b) => {
                send(Some(Msg::ReceiveWebSocketMessage(b)));
            }
        },
    )));

    client
}

// ------ ------
//     Init
// ------ ------

#[allow(clippy::needless_pass_by_value)]
fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(|subs::UrlRequested(url, url_request)| {
        if url.path().starts_with(&["auth".to_owned()]) {
            url_request.handled();
        }
    });

    orders
        .stream(streams::interval(1000, || Msg::OnRenderTick))
        .subscribe(Msg::UrlChanged);
    let document = Document::load();
    let page = Page::init(url.clone(), &document, orders);

    let web_socket = create_websocket(orders);

    Model {
        global: GlobalModel {
            document,
            base_url: url.to_hash_base_url(),
            web_socket,
            web_socket_reconnector: None,
        },
        page,
    }
}

// ------ ------
//     Model
// ------ ------

pub struct GlobalModel {
    document: Document,
    base_url: Url,
    web_socket: EventClient,
    web_socket_reconnector: Option<StreamHandle>,
}

pub struct Model {
    global: GlobalModel,
    page: Page,
}

// ------ ------
//     Urls
// ------ ------

struct_urls!();
impl<'a> Urls<'a> {
    #[must_use]
    pub fn home(self) -> Url {
        self.base_url()
    }

    #[must_use]
    pub fn auth(self) -> Url {
        self.base_url().add_hash_path_part(AUTH)
    }

    #[must_use]
    pub fn settings(self) -> Url {
        self.base_url().add_hash_path_part(SETTINGS)
    }

    #[must_use]
    pub fn view_task(self, id: &TaskId) -> Url {
        self.base_url()
            .add_hash_path_part(VIEW_TASK)
            .add_hash_path_part(id.to_string())
    }
}

// ------ ------
//     Pages
// ------ ------

#[derive(Debug)]
enum Page {
    Home(pages::home::Model),
    ViewTask(pages::view_task::Model),
    Auth(pages::auth::Model),
    Settings(pages::settings::Model),
}

impl Page {
    fn init(mut url: Url, document: &Document, orders: &mut impl Orders<Msg>) -> Self {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => match url.next_hash_path_part() {
                Some(id) => match TaskId::try_from(id) {
                    Ok(id) => {
                        if document.get_task(&id).is_some() {
                            Self::ViewTask(pages::view_task::init(id, orders))
                        } else {
                            Self::Home(pages::home::init(orders))
                        }
                    }
                    Err(_) => Self::Home(pages::home::init(orders)),
                },
                None => Self::Home(pages::home::init(orders)),
            },
            Some(AUTH) => Self::Auth(pages::auth::init(orders)),
            Some(SETTINGS) => Self::Settings(pages::settings::init()),
            None | Some(_) => Self::Home(pages::home::init(orders)),
        }
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    SelectTask(Option<TaskId>),
    CreateTask,
    OnRenderTick,
    UrlChanged(subs::UrlChanged),
    Home(pages::home::Msg),
    ViewTask(pages::view_task::Msg),
    Auth(pages::auth::Msg),
    Settings(pages::settings::Msg),

    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketFailed,
    ReconnectWebSocket(usize),
    SendWebSocketMessage(Vec<u8>),
    ReceiveWebSocketMessage(Vec<u8>),

    GoAuth,
    GoSettings,
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            orders.request_url(Urls::new(&model.global.base_url).home());
        }
        Msg::SelectTask(Some(id)) => {
            orders.request_url(Urls::new(&model.global.base_url).view_task(&id));
        }
        Msg::CreateTask => {
            let id = model.global.document.new_task();
            orders.request_url(Urls::new(&model.global.base_url).view_task(&id));
        }
        Msg::GoAuth => {
            orders.request_url(Urls::new(&model.global.base_url).auth());
        }
        Msg::GoSettings => {
            orders.request_url(Urls::new(&model.global.base_url).settings());
        }
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.page = Page::init(url, &model.global.document, orders);
        }
        Msg::ViewTask(msg) => {
            if let Page::ViewTask(lm) = &mut model.page {
                pages::view_task::update(msg, &mut model.global, lm, orders);
            }
        }
        Msg::Auth(msg) => {
            if let Page::Auth(lm) = &mut model.page {
                pages::auth::update(msg, &mut model.global, lm, orders);
            }
        }
        Msg::Settings(msg) => {
            if let Page::Settings(lm) = &mut model.page {
                pages::settings::update(msg, &mut model.global, lm, orders);
            }
        }
        Msg::Home(msg) => {
            if let Page::Home(lm) = &mut model.page {
                pages::home::update(msg, lm, orders);
            }
        }
        Msg::WebSocketOpened => {
            model.global.web_socket_reconnector = None;
            log!("WebSocket connection is open now");
        }
        Msg::WebSocketClosed(close_event) => {
            log!("==================");
            log!("WebSocket connection was closed:");
            log!("Clean:", close_event.was_clean());
            log!("Code:", close_event.code());
            log!("Reason:", close_event.reason());
            log!("==================");

            if !close_event.was_clean() {
                // don't retry this
                // TODO: filter it up in the UI later
                model.global.web_socket_reconnector = None;
            }
        }
        Msg::WebSocketFailed => {
            log!("WebSocket failed");
            if model.global.web_socket_reconnector.is_none() {
                model.global.web_socket_reconnector = Some(
                    orders.stream_with_handle(streams::backoff(None, Msg::ReconnectWebSocket)),
                );
            }
        }
        Msg::ReconnectWebSocket(retries) => {
            log!("Reconnect attempt:", retries);
            model.global.web_socket = create_websocket(orders);
        }
        Msg::SendWebSocketMessage(message) => {
            if let Err(err) = model.global.web_socket.send_binary(message) {
                log!("Failed to send websocket message:", err);
            }
        }
        Msg::ReceiveWebSocketMessage(message) => {
            match SyncMessage::try_from(&message) {
                Ok(message) => match message {
                    SyncMessage::Message(m) => {
                        log!("Applying sync message");
                        model.global.document.receive_sync_message(&m);
                    }
                },
                Err(err) => {
                    log!(format!(
                        "Failed to decode websocket sync message: {:?}",
                        err
                    ));
                }
            }
            log!("Received ws message");
        }
    }
    model.global.document.save();
    if let Some(msg) = model.global.document.generate_sync_message() {
        let sync_message = SyncMessage::Message(msg);
        match Vec::try_from(sync_message) {
            Ok(bytes) => {
                log!("sending sync message");
                orders.send_msg(Msg::SendWebSocketMessage(bytes));
            }
            Err(err) => {
                log!(format!("Failed to serialize sync message {:?}", err));
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "container", "mx-auto"],
        view_titlebar(model),
        match &model.page {
            Page::Home(lm) => pages::home::view(&model.global, lm),
            Page::ViewTask(lm) => pages::view_task::view(&model.global, lm),
            Page::Auth(lm) => pages::auth::view(&model.global, lm),
            Page::Settings(lm) => pages::settings::view(&model.global, lm),
        },
    ]
}

fn view_titlebar(model: &Model) -> Node<Msg> {
    let signed_in = Provider::load_from_session().is_some();
    let account_string = if signed_in { "Account" } else { "Sign in" };

    let connection_string = if signed_in {
        match *model.global.web_socket.status.borrow() {
            wasm_sockets::ConnectionStatus::Connecting => "Connecting",
            wasm_sockets::ConnectionStatus::Connected => "Connected",
            wasm_sockets::ConnectionStatus::Error
            | wasm_sockets::ConnectionStatus::Disconnected => "Disconnected",
        }
    } else {
        "Sign in before syncing"
    };
    let connection = span![
        attrs! {
            At::Title => "Click to reconnect",
        },
        connection_string
    ];
    div![
        C!["flex", "flex-row", "justify-between"],
        div![
            C!["flex", "flex-row", "justify-start"],
            a![
                C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300",],
                attrs! {At::Href => "#"},
                "TaskNet"
            ]
        ],
        nav![
            C!["flex", "flex-row", "justify-end"],
            view_button(
                connection,
                Msg::ReconnectWebSocket(0),
                &ButtonOptions {
                    disabled: !signed_in
                }
            ),
            view_button_str(account_string, Msg::GoAuth),
            view_button_str("Settings", Msg::GoSettings),
            view_button_str("Create", Msg::CreateTask),
        ]
    ]
}

// ------ ------
//     Start
// ------ ------

pub fn main() {
    App::start("app", init, update, view);
}
