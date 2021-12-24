use apply::Apply;
use automerge_backend::SyncMessage;
use chrono::Utc;
use derivative::Derivative;
use kratos_api::models::{SelfServiceLogoutUrl, Session};
use seed::browser::web_socket::State;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod components;
mod document;
mod filters;
mod pages;
mod settings;
mod task;
mod urgency;

use components::view_button;
use document::Document;
use filters::Filters;
use settings::Settings;
use task::{Recur, Status, Task};
use tasknet_sync::Message;

use crate::components::view_link;

const VIEW_TASK: &str = "view";
const SETTINGS: &str = "settings";
const ACCOUNT: &str = "account";
const LOGIN: &str = "login";
const REGISTRATION: &str = "registration";
const SERVER_PEER_ID: &[u8] = b"server";
const SETTINGS_STORAGE_KEY: &str = "tasknet-settings";

fn ws_url(doc_id: uuid::Uuid) -> String {
    let location = window().location();
    format!(
        "wss://{}{}/sync?doc_id={}",
        location.host().unwrap(),
        location.pathname().unwrap(),
        doc_id
    )
}

fn create_websocket(orders: &impl Orders<Msg>, doc_id: uuid::Uuid) -> WebSocket {
    let msg_sender = orders.msg_sender();

    WebSocket::builder(ws_url(doc_id), orders)
        .on_open(|| Msg::WebSocketOpened)
        .on_message(|message| {
            spawn_local(async move {
                let bytes = message
                    .bytes()
                    .await
                    .expect("WebsocketError on binary data");

                let message = Message::from(bytes);
                match message {
                    Message::SyncMessage(sync_message) => {
                        msg_sender(Some(Msg::ReceiveSyncMessage(sync_message)))
                    }
                }
            })
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
    orders.subscribe(|subs::UrlRequested(url, url_request)| {
        if url.path().contains(&"kratos".to_string()) {
            url_request.handled()
        }
    });
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
            log!("Error registering service worker:", e)
        }
    });
    orders
        .stream(streams::interval(1000, || Msg::OnRenderTick))
        .stream(streams::interval(60000, || Msg::OnRecurTick))
        .stream(streams::interval(60000, || Msg::BackendCompactTick))
        .stream(streams::interval(60000, || Msg::BackendPeriodicSyncTick))
        .subscribe(Msg::UrlChanged);
    let document = Document::new();
    let settings = match LocalStorage::get(SETTINGS_STORAGE_KEY) {
        Ok(settings) => settings,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse settings: {:?}", err)
        }
        Err(_) => {
            let settings = Settings::default();
            LocalStorage::insert(SETTINGS_STORAGE_KEY, &settings).unwrap();
            settings
        }
    };
    let web_socket = create_websocket(orders, settings.document_id);
    orders.send_msg(Msg::WhoAmI);
    let global_model = GlobalModel {
        document,
        base_url: url.to_hash_base_url(),
        settings,
        is_logged_in: false,
        logout_url: String::new(),
        web_socket,
        web_socket_reconnector: None,
    };
    let page = Page::init(url, &global_model, orders);
    Model {
        global: global_model,
        page,
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Derivative)]
#[derivative(Debug)]
pub struct GlobalModel {
    #[derivative(Debug = "ignore")]
    document: Document,
    base_url: Url,
    settings: Settings,
    is_logged_in: bool,
    logout_url: String,
    // TODO: move to SyncModel,
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
    pub fn view_task(self, uuid: &uuid::Uuid) -> Url {
        self.base_url()
            .add_hash_path_part(VIEW_TASK)
            .add_hash_path_part(uuid.to_string())
    }

    #[must_use]
    pub fn settings(self) -> Url {
        self.base_url().add_hash_path_part(SETTINGS)
    }

    #[must_use]
    pub fn account(self) -> Url {
        self.base_url().add_hash_path_part(ACCOUNT)
    }

    #[must_use]
    pub fn login(self) -> Url {
        self.base_url().add_hash_path_part(LOGIN)
    }

    #[must_use]
    pub fn registration(self) -> Url {
        self.base_url().add_hash_path_part(REGISTRATION)
    }
}

// ------ ------
//     Pages
// ------ ------

#[derive(Debug)]
enum Page {
    Home(pages::home::Model),
    ViewTask(pages::view_task::Model),
    Settings(pages::settings::Model),
    Account(pages::account::Model),
    Login(pages::login::Model),
    Registration(pages::registration::Model),
}

impl Page {
    fn init(mut url: Url, global_model: &GlobalModel, orders: &mut impl Orders<Msg>) -> Self {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => match url.next_hash_path_part() {
                Some(uuid) => {
                    if let Ok(uuid) = uuid::Uuid::parse_str(uuid) {
                        if let Some(task) = global_model.document.task(&uuid) {
                            Self::ViewTask(pages::view_task::init(uuid, task.clone(), orders))
                        } else {
                            Self::ViewTask(pages::view_task::init(uuid, Task::new(), orders))
                        }
                    } else {
                        Self::Home(pages::home::init())
                    }
                }
                None => Self::Home(pages::home::init()),
            },
            Some(SETTINGS) => Self::Settings(pages::settings::init(global_model)),
            Some(ACCOUNT) => Self::Account(pages::account::init(global_model, orders)),
            Some(LOGIN) => Self::Login(pages::login::init(global_model, orders)),
            Some(REGISTRATION) => {
                Self::Registration(pages::registration::init(global_model, orders))
            }
            None | Some(_) => Self::Home(pages::home::init()),
        }
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Debug, Clone)]
pub enum Msg {
    SelectTask(Option<uuid::Uuid>),
    CreateTask,
    ViewSettings,
    GetLogoutUrl,
    SetLogoutUrl(String),
    WhoAmI,
    SetLoggedIn(bool),
    OnRenderTick,
    OnRecurTick,
    BackendCompactTick,
    BackendPeriodicSyncTick,
    SendSyncMessage,
    ReceiveSyncMessage(SyncMessage),
    WebSocketOpened,
    WebSocketClosed(CloseEvent),
    WebSocketFailed,
    ReconnectWebSocket(usize),
    SendWebSocketMessage(Message),
    UrlChanged(subs::UrlChanged),
    ApplyChange(automerge_protocol::Change),
    ApplyPatch(automerge_protocol::Patch),
    ChangeDocument,
    Home(pages::home::Msg),
    ViewTask(pages::view_task::Msg),
    Settings(pages::settings::Msg),
    Account(pages::account::Msg),
    Login(pages::login::Msg),
    Registration(pages::registration::Msg),
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            orders.request_url(Urls::new(&model.global.base_url).home());
        }
        Msg::SelectTask(Some(uuid)) => {
            orders.request_url(Urls::new(&model.global.base_url).view_task(&uuid));
        }
        Msg::CreateTask => {
            let id = uuid::Uuid::new_v4();
            orders.request_url(Urls::new(&model.global.base_url).view_task(&id));
        }
        Msg::ViewSettings => {
            orders.request_url(Urls::new(&model.global.base_url).settings());
        }
        Msg::GetLogoutUrl => {
            orders.perform_cmd(async move {
                let response = fetch(
                    Request::new("/kratos/self-service/logout/browser")
                        .header(Header::custom("Accept", "application/json")),
                )
                .await
                .expect("HTTP request failed");
                let value = response.json::<SelfServiceLogoutUrl>().await.unwrap();
                log!(value);
                Msg::SetLogoutUrl(value.logout_url)
            });
        }
        Msg::SetLogoutUrl(url) => {
            model.global.logout_url = url;
        }
        Msg::WhoAmI => {
            orders.perform_cmd(async move {
                let response = fetch(
                    Request::new("/kratos/sessions/whoami")
                        .header(Header::custom("Accept", "application/json")),
                )
                .await
                .expect("HTTP request failed");
                match response.check_status() {
                    Ok(response) => match response.json::<Session>().await {
                        Ok(value) => {
                            log!(value);
                            Msg::SetLoggedIn(value.active.unwrap_or_default())
                        }
                        Err(err) => {
                            log!(err);
                            Msg::SetLoggedIn(false)
                        }
                    },
                    Err(err) => {
                        log!(err);
                        Msg::SetLoggedIn(false)
                    }
                }
            });
        }
        Msg::SetLoggedIn(logged_in) => {
            if logged_in {
                orders.send_msg(Msg::GetLogoutUrl);
            }
            model.global.is_logged_in = logged_in;
        }
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::OnRecurTick => {
            let tasks = model.global.document.tasks();
            let recurring: Vec<_> = tasks
                .iter()
                .filter(|(_, t)| t.status() == &Status::Recurring)
                .collect();
            let mut new_tasks = Vec::new();
            for (id, task) in recurring {
                let latest_child: Option<&Task> = tasks
                    .values()
                    .filter(|t| t.parent().as_ref().map_or(false, |p| *p == *id))
                    .fold(None, |acc, t| {
                        if let Some(acc) = acc {
                            if **t.entry() > **acc.entry() {
                                Some(t)
                            } else {
                                Some(acc)
                            }
                        } else {
                            Some(t)
                        }
                    });
                if let Some(child) = latest_child {
                    // if child's entry is older than the recurring duration, create a new child
                    let time_since_last = Utc::now() - **child.entry();
                    if time_since_last > task.recur().as_ref().unwrap().duration() {
                        let new_child = task.new_child(**id);
                        new_tasks.push(new_child)
                    }
                } else {
                    let new_child = task.new_child(**id);
                    new_tasks.push(new_child)
                }
            }
            for (i, t) in new_tasks {
                let msg = model.global.document.set_task(i, t);
                if let Some(msg) = msg {
                    orders.send_msg(msg);
                }
            }
        }
        Msg::BackendCompactTick => {
            log!("compacting");
            model.global.document.backend.compact(&[]).unwrap();
            log!("compacted");
        }
        Msg::BackendPeriodicSyncTick => {
            log!("periodic sync");
            orders.skip().send_msg(Msg::SendSyncMessage);
        }
        Msg::SendSyncMessage => {
            let sync_message = model
                .global
                .document
                .backend
                .generate_sync_message(SERVER_PEER_ID.to_vec())
                .unwrap();
            if let Some(sync_message) = sync_message {
                orders
                    .skip()
                    .send_msg(Msg::SendWebSocketMessage(Message::SyncMessage(
                        sync_message,
                    )));
                log!("send sync message");
            }
        }
        Msg::ReceiveSyncMessage(sync_message) => {
            let patch = model
                .global
                .document
                .backend
                .receive_sync_message(SERVER_PEER_ID.to_vec(), sync_message)
                .unwrap();
            if let Some(patch) = patch {
                orders.skip().send_msg(Msg::ApplyPatch(patch));
            }
            // we may now want to send another message so give that a call
            orders.skip().send_msg(Msg::SendSyncMessage);
        }
        Msg::WebSocketOpened => {
            model.global.web_socket_reconnector = None;
            // reset the sync state to ensure we start clean and avoid loops
            model
                .global
                .document
                .backend
                .reset_sync_state(SERVER_PEER_ID);
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
            model.global.web_socket = create_websocket(orders, model.global.settings.document_id);
        }
        Msg::SendWebSocketMessage(message) => {
            let bytes = Vec::<u8>::from(message);
            model.global.web_socket.send_bytes(&bytes).unwrap();
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.page = Page::init(url, &model.global, orders)
        }
        Msg::ApplyChange(change) => {
            let patch = model
                .global
                .document
                .backend
                .apply_local_change(change)
                .unwrap();
            orders.skip().send_msg(Msg::ApplyPatch(patch));
            orders.skip().send_msg(Msg::SendSyncMessage);
        }
        Msg::ApplyPatch(patch) => {
            log!("apply patch");
            model.global.document.inner.apply_patch(patch).unwrap();
            log!("applied patch")
        }
        Msg::ChangeDocument => {
            // given a new document id we now need to load the automerge document into memory and
            // reestablish the sync session
            model.global.document = Document::new();
            model.global.web_socket = create_websocket(orders, model.global.settings.document_id);
        }
        Msg::ViewTask(msg) => {
            if let Page::ViewTask(lm) = &mut model.page {
                pages::view_task::update(msg, &mut model.global, lm, orders)
            }
        }
        Msg::Home(msg) => {
            if let Page::Home(lm) = &mut model.page {
                pages::home::update(msg, lm, orders)
            }
        }
        Msg::Settings(msg) => {
            if let Page::Settings(lm) = &mut model.page {
                pages::settings::update(msg, &mut model.global, lm, orders)
            }
        }
        Msg::Account(msg) => {
            if let Page::Account(lm) = &mut model.page {
                pages::account::update(msg, &mut model.global, lm, orders)
            }
        }
        Msg::Login(msg) => {
            if let Page::Login(lm) = &mut model.page {
                pages::login::update(msg, &mut model.global, lm, orders)
            }
        }
        Msg::Registration(msg) => {
            if let Page::Registration(lm) = &mut model.page {
                pages::registration::update(msg, &mut model.global, lm, orders)
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
            Page::Settings(lm) => pages::settings::view(&model.global, lm),
            Page::Account(lm) => pages::account::view(&model.global, lm),
            Page::Login(lm) => pages::login::view(&model.global, lm),
            Page::Registration(lm) => pages::registration::view(&model.global, lm),
        },
    ]
}

fn view_titlebar(model: &Model) -> Node<Msg> {
    let is_home = matches!(model.page, Page::Home(_));
    let is_settings = matches!(model.page, Page::Settings(_));
    div![
        C!["flex", "flex-row", "justify-between"],
        div![
            C!["flex", "flex-row", "justify-start"],
            view_button("Tasknet", Msg::SelectTask(None), false),
            if model.global.is_logged_in {
                vec![
                    view_link("Account", "/kratos/self-service/settings/browser", false),
                    view_link(
                        "Logout",
                        &model.global.logout_url,
                        model.global.logout_url.is_empty(),
                    ),
                ]
            } else {
                vec![
                    view_link("Login", "/kratos/self-service/login/browser", false),
                    view_link(
                        "Register",
                        "/kratos/self-service/registration/browser",
                        false,
                    ),
                ]
            },
        ],
        div![
            C!["my-auto"],
            match model.global.web_socket.state() {
                State::Connecting => "Connecting",
                State::Open => "Online",
                State::Closing => "Offline",
                State::Closed => "Offline",
                _ => "Offline",
            }
        ],
        nav![
            C!["flex", "flex-row", "justify-end"],
            if is_home {
                view_button(
                    "Toggle Filters",
                    Msg::Home(pages::home::Msg::ToggleShowFilters),
                    false,
                )
            } else {
                empty!()
            },
            if !is_settings {
                view_button("Settings", Msg::ViewSettings, false)
            } else {
                empty!()
            },
            view_button("Create", Msg::CreateTask, false),
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
