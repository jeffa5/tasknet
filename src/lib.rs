#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::collections::HashMap;

use apply::Apply;
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod components;
mod filters;
mod pages;
mod task;
mod urgency;

use components::view_button;
use filters::Filters;
use task::{Recur, Status, Task};

const VIEW_TASK: &str = "view";
const TASKS_STORAGE_KEY: &str = "tasknet-tasks";

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
    let tasks = match LocalStorage::get(TASKS_STORAGE_KEY) {
        Ok(tasks) => tasks,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse tasks: {:?}", err)
        }
        Err(_) => HashMap::new(),
    };
    let page = Page::init(url.clone(), &tasks, orders);
    Model {
        global: GlobalModel {
            tasks,
            base_url: url.to_hash_base_url(),
        },
        page,
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
pub struct GlobalModel {
    tasks: HashMap<uuid::Uuid, Task>,
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
        self.base_url().add_path_part("tasknet")
    }

    #[must_use]
    pub fn view_task(self, uuid: &uuid::Uuid) -> Url {
        self.base_url()
            .add_path_part("tasknet")
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
    fn init(
        mut url: Url,
        tasks: &HashMap<uuid::Uuid, Task>,
        orders: &mut impl Orders<Msg>,
    ) -> Self {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => match url.next_hash_path_part() {
                Some(uuid) => {
                    if let Ok(uuid) = uuid::Uuid::parse_str(uuid) {
                        if tasks.get(&uuid).is_some() {
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
    Home(pages::home::Msg),
    ViewTask(pages::view_task::Msg),
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            Urls::new(&model.global.base_url).home().go_and_push();
        }
        Msg::SelectTask(Some(uuid)) => {
            Urls::new(&model.global.base_url)
                .view_task(&uuid)
                .go_and_push();
            model.page = Page::ViewTask(pages::view_task::init(uuid, orders))
        }
        Msg::CreateTask => {
            let task = Task::new();
            let id = task.uuid();
            model.global.tasks.insert(task.uuid(), task);
            model.page = Page::ViewTask(pages::view_task::init(id, orders));
            Urls::new(&model.global.base_url)
                .view_task(&id)
                .go_and_push();
        }
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::OnRecurTick => {
            let recurring: Vec<_> = model
                .global
                .tasks
                .values()
                .filter(|t| t.status() == &Status::Recurring)
                .collect();
            let mut new_tasks = Vec::new();
            for r in recurring {
                let mut children: Vec<_> = model
                    .global
                    .tasks
                    .values()
                    .filter(|t| t.parent().map_or(false, |p| p == r.uuid()))
                    .collect();
                children.sort_by_key(|c| c.entry());
                let last_child = children.last();
                if let Some(child) = last_child {
                    // if child's entry is older than the recurring duration, create a new child
                    if chrono::offset::Utc::now() - *child.entry()
                        > r.recur().as_ref().unwrap().duration()
                    {
                        log!("old enough");
                        let new_child = r.new_child();
                        new_tasks.push(new_child)
                    }
                } else {
                    let new_child = r.new_child();
                    new_tasks.push(new_child)
                }
            }
            for t in new_tasks {
                model.global.tasks.insert(t.uuid(), t);
            }
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.page = Page::init(url, &model.global.tasks, orders)
        }
        Msg::ImportTasks => match window().prompt_with_message("Paste the tasks json here") {
            Ok(Some(content)) => {
                match serde_json::from_str::<HashMap<uuid::Uuid, Task>>(&content) {
                    Ok(tasks) => {
                        for (id, task) in tasks {
                            model.global.tasks.insert(id, task);
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
            let json = serde_json::to_string(&model.global.tasks);
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
                pages::view_task::update(msg, &mut model.global, lm, orders)
            }
        }
        Msg::Home(msg) => {
            if let Page::Home(lm) = &mut model.page {
                pages::home::update(msg, lm, orders)
            }
        }
    }
    LocalStorage::insert(TASKS_STORAGE_KEY, &model.global.tasks)
        .expect("save tasks to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
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
