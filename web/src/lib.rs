#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::{collections::HashMap, convert::TryFrom};

use apply::Apply;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod auth;
mod components;
mod document;
mod filters;
mod pages;
mod task;
mod urgency;

use components::{view_button, view_button_str};
use document::Document;
use filters::Filters;
use sync::SyncMessage;
use task::{Recur, Status, Task, TaskId};

const VIEW_TASK: &str = "view";
const AUTH: &str = "auth";

fn ws_url() -> String {
    let location = window().location();
    format!(
        "ws://{}{}sync",
        location.host().unwrap(),
        location.pathname().unwrap(),
    )
}

fn create_websocket(orders: &impl Orders<Msg>) -> WebSocket {
    let msg_sender = orders.msg_sender();

    WebSocket::builder(ws_url(), orders)
        .on_open(|| Msg::WebSocketOpened)
        .on_message(|message| {
            spawn_local(async move {
                let bytes = message
                    .bytes()
                    .await
                    .expect("WebsocketError on binary data");

                msg_sender(Some(Msg::ReceiveWebSocketMessage(bytes)));
            });
        })
        .on_close(Msg::WebSocketClosed)
        .on_error(|| Msg::WebSocketFailed)
        .build_and_open()
        .unwrap()
}

// ------ ------
//     Init
// ------ ------

#[allow(clippy::needless_pass_by_value)]
fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let url_clone = url.clone();
    orders.perform_cmd(async move {
        let res = window()
            .navigator()
            .service_worker()
            .register(&format!(
                "{}/{}",
                url_clone.path().join("/"),
                "service-worker.js"
            ))
            .apply(JsFuture::from)
            .await;
        if let Err(e) = res {
            log!("Error registering service worker:", e);
        }
    });

    orders.subscribe(|subs::UrlRequested(url, url_request)| {
        if url.path().starts_with(&["auth".to_owned()]) {
            url_request.handled();
        }
    });

    orders
        .stream(streams::interval(1000, || Msg::OnRenderTick))
        .stream(streams::interval(60000, || Msg::OnRecurTick))
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

#[derive(Debug)]
pub struct GlobalModel {
    document: Document,
    base_url: Url,
    web_socket: WebSocket,
    web_socket_reconnector: Option<StreamHandle>,
}

#[derive(Debug)]
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
}

impl Page {
    fn init(mut url: Url, document: &Document, orders: &mut impl Orders<Msg>) -> Self {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => url.next_hash_path_part().map_or_else(
                || Self::Home(pages::home::init()),
                |id| {
                    TaskId::try_from(id).map_or_else(
                        |_| Self::Home(pages::home::init()),
                        |id| {
                            if document.get_task(&id).is_some() {
                                Self::ViewTask(pages::view_task::init(id, orders))
                            } else {
                                Self::Home(pages::home::init())
                            }
                        },
                    )
                },
            ),
            Some(AUTH) => Self::Auth(pages::auth::init()),
            None | Some(_) => Self::Home(pages::home::init()),
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
    OnRecurTick,
    UrlChanged(subs::UrlChanged),
    ImportTasks,
    ExportTasks,
    Home(pages::home::Msg),
    ViewTask(pages::view_task::Msg),
    Auth(pages::auth::Msg),

    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketFailed,
    ReconnectWebSocket(usize),
    SendWebSocketMessage(Vec<u8>),
    ReceiveWebSocketMessage(Vec<u8>),

    GoAuth,
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
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::OnRecurTick => {
            let recurring: Vec<_> = model
                .global
                .document
                .tasks()
                .values()
                .filter(|t| t.status() == &Status::Recurring)
                .collect();
            let mut new_tasks = Vec::new();
            for r in recurring {
                let mut children: Vec<_> = model
                    .global
                    .document
                    .tasks()
                    .values()
                    .filter(|t| t.parent().as_ref().map_or(false, |p| p == r.id()))
                    .collect();
                children.sort_by_key(|c| c.entry());
                let last_child = children.last();
                if let Some(child) = last_child {
                    // if child's entry is older than the recurring duration, create a new child
                    if chrono::offset::Utc::now() - child.entry().0
                        > r.recur().as_ref().unwrap().duration()
                    {
                        log!("old enough");
                        let new_child = r.new_child();
                        new_tasks.push(new_child);
                    }
                } else {
                    let new_child = r.new_child();
                    new_tasks.push(new_child);
                }
            }
            for t in new_tasks {
                model.global.document.update_task(t);
            }
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.page = Page::init(url, &model.global.document, orders);
        }
        Msg::ImportTasks => match window().prompt_with_message("Paste the tasks json here") {
            Ok(Some(content)) => match serde_json::from_str::<HashMap<TaskId, Task>>(&content) {
                Ok(tasks) => {
                    for task in tasks.into_values() {
                        model.global.document.update_task(task);
                    }
                }
                Err(e) => {
                    log!(e);
                    window()
                        .alert_with_message("Failed to import tasks")
                        .unwrap_or_else(|e| log!(e));
                }
            },
            Ok(None) => {}
            Err(e) => {
                log!(e);
                window()
                    .alert_with_message("Failed to create prompt")
                    .unwrap_or_else(|e| log!(e));
            }
        },
        Msg::ExportTasks => {
            let json = serde_json::to_string(&model.global.document.tasks());
            match json {
                Ok(json) => {
                    window()
                        .prompt_with_message_and_default("Copy this", &json)
                        .unwrap_or_else(|e| {
                            log!(e);
                            None
                        });
                }
                Err(e) => log!(e),
            }
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

            // Chrome doesn't invoke `on_error` when the connection is lost.
            if !close_event.was_clean() && model.global.web_socket_reconnector.is_none() {
                model.global.web_socket_reconnector = Some(
                    orders.stream_with_handle(streams::backoff(None, Msg::ReconnectWebSocket)),
                );
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
            model.global.web_socket.send_bytes(&message).unwrap();
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
                    log!("Failed to decode websocket sync message", err);
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
                log!("Failed to serialize sync message", err);
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
        },
    ]
}

fn view_titlebar(model: &Model) -> Node<Msg> {
    let account_string = if auth::provider().is_some() {
        "Account"
    } else {
        "Sign in"
    };

    let connection_string = match model.global.web_socket.state() {
        web_sys::TcpReadyState::Connecting => "Connecting",
        web_sys::TcpReadyState::Open => "Connected",
        web_sys::TcpReadyState::Closing | web_sys::TcpReadyState::Closed => "Disconnected",
        _ => todo!(),
    };
    let connection = span![
        attrs! {At::Title => "Click to reconnect"},
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
            view_button_str(account_string, Msg::GoAuth),
            view_button(connection, Msg::ReconnectWebSocket(0)),
            view_button_str("Import Tasks", Msg::ImportTasks),
            view_button_str("Export Tasks", Msg::ExportTasks),
            view_button_str("Create", Msg::CreateTask),
        ]
    ]
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
