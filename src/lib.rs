#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::collections::HashMap;

use apply::Apply;
use automerge::{Backend, Frontend};
use derivative::Derivative;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod components;
mod document;
mod filters;
mod pages;
mod task;
mod urgency;

use components::view_button;
use document::Document;
use filters::Filters;
use task::{Recur, Status, Task};

const VIEW_TASK: &str = "view";

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
            log!("Error registering service worker:", e)
        }
    });
    orders
        .stream(streams::interval(1000, || Msg::OnRenderTick))
        .stream(streams::interval(60000, || Msg::OnRecurTick))
        .subscribe(Msg::UrlChanged);
    let document = Document::new();
    let page = Page::init(url.clone(), &document, orders);
    Model {
        global: GlobalModel {
            document,
            base_url: url.to_hash_base_url(),
        },
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
}

// ------ ------
//     Pages
// ------ ------

#[derive(Debug)]
enum Page {
    Home(pages::home::Model),
    ViewTask(pages::view_task::Model),
}

impl Page {
    fn init(mut url: Url, document: &Document, orders: &mut impl Orders<Msg>) -> Self {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => match url.next_hash_path_part() {
                Some(uuid) => {
                    if let Ok(uuid) = uuid::Uuid::parse_str(uuid) {
                        if document.task(&uuid).is_some() {
                            Self::ViewTask(pages::view_task::init(uuid, orders))
                        } else {
                            Self::Home(pages::home::init())
                        }
                    } else {
                        Self::Home(pages::home::init())
                    }
                }
                None => Self::Home(pages::home::init()),
            },
            None | Some(_) => Self::Home(pages::home::init()),
        }
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    SelectTask(Option<uuid::Uuid>),
    CreateTask,
    OnRenderTick,
    OnRecurTick,
    UrlChanged(subs::UrlChanged),
    ImportTasks,
    ExportTasks,
    ApplyChange(automerge_protocol::UncompressedChange),
    ApplyPatch(automerge_protocol::Patch),
    Home(pages::home::Msg),
    ViewTask(pages::view_task::Msg),
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
            model.global.document.add_task(id);
            orders.request_url(Urls::new(&model.global.base_url).view_task(&id));
        }
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::OnRecurTick => {
            let tasks = model.global.document.tasks();
            let recurring: Vec<_> = tasks
                .iter()
                .filter(|(_, t)| t.status() == &Status::Recurring)
                .collect();
            let mut new_tasks = Vec::new();
            for r in recurring {
                let mut children: Vec<_> = tasks
                    .iter()
                    .filter(|(i, t)| t.parent().map_or(false, |p| p == **i))
                    .collect();
                children.sort_by_key(|c| c.1.entry());
                let last_child = children.last();
                if let Some(child) = last_child {
                    // if child's entry is older than the recurring duration, create a new child
                    if chrono::offset::Utc::now() - *child.1.entry()
                        > r.1.recur().as_ref().unwrap().duration()
                    {
                        log!("old enough");
                        let new_child = r.1.new_child(*r.0);
                        new_tasks.push(new_child)
                    }
                } else {
                    let new_child = r.1.new_child(*r.0);
                    new_tasks.push(new_child)
                }
            }
            for (i, t) in new_tasks {
                // model.global.tasks.insert(i, t);
            }
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.page = Page::init(url, &model.global.document, orders)
        }
        Msg::ImportTasks => match window().prompt_with_message("Paste the tasks json here") {
            Ok(Some(content)) => {
                match serde_json::from_str::<HashMap<uuid::Uuid, Task>>(&content) {
                    Ok(tasks) => {
                        for (id, task) in tasks {
                            // model.global.tasks.insert(id, task);
                        }
                    }
                    Err(e) => {
                        log!(e);
                        window()
                            .alert_with_message("Failed to import tasks")
                            .unwrap_or_else(|e| log!(e));
                    }
                }
            }
            Ok(None) => {}
            Err(e) => {
                log!(e);
                window()
                    .alert_with_message("Failed to create prompt")
                    .unwrap_or_else(|e| log!(e));
            }
        },
        Msg::ExportTasks => {
            // let json = serde_json::to_string(&model.global.document);
            // match json {
            //     Ok(json) => {
            //         window()
            //             .prompt_with_message_and_default("Copy this", &json)
            //             .unwrap_or_else(|e| {
            //                 log!(e);
            //                 None
            //             });
            //     }
            //     Err(e) => log!(e),
            // }
        }
        Msg::ApplyChange(change) => {
                let (patch, _) = model.global.document.backend.apply_local_change(change).unwrap();
            orders.skip().send_msg(
                Msg::ApplyPatch(patch)
            );
        }
        Msg::ApplyPatch(patch) => {
            model.global.document.frontend.apply_patch(patch).unwrap()
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
    }
    LocalStorage::insert(document::TASKS_STORAGE_KEY, &model.global.document.save())
        .expect("save tasks to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["flex", "flex-col", "container", "mx-auto"],
        view_titlebar(),
        match &model.page {
            Page::Home(lm) => pages::home::view(&model.global, lm),
            Page::ViewTask(lm) => pages::view_task::view(&model.global, lm),
        },
    ]
}

fn view_titlebar() -> Node<Msg> {
    div![
        C!["flex", "flex-row", "justify-between"],
        div![
            C!["flex", "flex-row", "justify-start"],
            a![
                C!["bg-gray-200", "py-2", "px-4", "m-2", "hover:bg-gray-300",],
                attrs! {At::Href => "/tasknet"},
                "TaskNet"
            ]
        ],
        nav![
            C!["flex", "flex-row", "justify-end"],
            view_button("Import Tasks", Msg::ImportTasks),
            view_button("Export Tasks", Msg::ExportTasks),
            view_button("Create", Msg::CreateTask),
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
