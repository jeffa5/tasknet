#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::{collections::HashMap, convert::TryFrom};

use apply::Apply;
use chrono::{Datelike, Timelike};
#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

mod components;
mod filters;
mod pages;
mod task;
mod urgency;

use components::view_button;
use filters::Filters;
use task::{Priority, Recur, RecurUnit, Status, Task};

const ESCAPE_KEY: &str = "Escape";

const VIEW_TASK: &str = "view";
const VIEW_TASK_SEARCH_KEY: &str = "viewtask";
const TASKS_STORAGE_KEY: &str = "tasknet-tasks";
const FILTERS_STORAGE_KEY: &str = "tasknet-filters";

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
        .stream(streams::window_event(Ev::KeyUp, |event| {
            let key_event: web_sys::KeyboardEvent = event.unchecked_into();
            match key_event.key().as_ref() {
                ESCAPE_KEY => Some(Msg::EscapeKey),
                _ => None,
            }
        }))
        .subscribe(Msg::UrlChanged);
    let tasks = match LocalStorage::get(TASKS_STORAGE_KEY) {
        Ok(tasks) => tasks,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse tasks: {:?}", err)
        }
        Err(_) => HashMap::new(),
    };
    let filters = match LocalStorage::get(FILTERS_STORAGE_KEY) {
        Ok(filters) => filters,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse filters: {:?}", err)
        }
        Err(_) => Filters::default(),
    };
    let page = Page::init(url, &tasks);
    Model {
        tasks,
        filters,
        base_url: url.to_base_url(),
        page,
    }
}

// ------ ------
//     Model
// ------ ------

#[derive(Debug)]
pub struct Model {
    tasks: HashMap<uuid::Uuid, Task>,
    filters: Filters,
    base_url: Url,
    page:Page,
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
    Home,
    ViewTask(pages::view_task::Model),
}

impl Page {
    fn init(url: Url, tasks: &HashMap<uuid::Uuid, Task>) -> Page {
        match url.next_hash_path_part() {
            Some(VIEW_TASK) => match url.next_hash_path_part() {
                Some(uuid) => {
                    if let Ok(uuid) = uuid::Uuid::parse_str(uuid) {
                        if tasks.get(&uuid).is_some() {
                            Page::ViewTask(pages::view_task::init(uuid))
                        } else {
                            Page::Home
                        }
                    } else {
                        Page::Home
                    }
                }
                _ => Page::Home,
            },
            None | Some(_) => Page::Home,
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
    FiltersStatusTogglePending,
    FiltersStatusToggleDeleted,
    FiltersStatusToggleCompleted,
    FiltersStatusToggleWaiting,
    FiltersStatusToggleRecurring,
    FiltersPriorityToggleNone,
    FiltersPriorityToggleLow,
    FiltersPriorityToggleMedium,
    FiltersPriorityToggleHigh,
    FiltersProjectChanged(String),
    FiltersTagsChanged(String),
    FiltersDescriptionChanged(String),
    FiltersReset,
    UrlChanged(subs::UrlChanged),
    EscapeKey,
    ImportTasks,
    ExportTasks,
    ViewTask(pages::view_task::Msg),
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SelectTask(None) => {
            Urls::new(&model.base_url).home().go_and_push();
            model.page = Page::Home
        }
        Msg::SelectTask(Some(uuid)) => {
            Urls::new(&model.base_url).view_task(&uuid).go_and_push();
            model.page = Page::ViewTask(pages::view_task::init(uuid))
        }
        Msg::CreateTask => {
            let task = Task::new();
            let id = task.uuid();
            model.tasks.insert(task.uuid(), task);
            model.selected_task = Some(id);
            Urls::new(&model.base_url).view_task(&id).go_and_push();
        }
        Msg::OnRenderTick => { /* just re-render to update the ages */ }
        Msg::OnRecurTick => {
            let recurring: Vec<_> = model
                .tasks
                .values()
                .filter(|t| t.status() == &Status::Recurring)
                .collect();
            let mut new_tasks = Vec::new();
            for r in recurring {
                let mut children: Vec<_> = model
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
                model.tasks.insert(t.uuid(), t);
            }
        }
        Msg::FiltersStatusTogglePending => {
            model.filters.status_pending = !model.filters.status_pending
        }
        Msg::FiltersStatusToggleDeleted => {
            model.filters.status_deleted = !model.filters.status_deleted
        }
        Msg::FiltersStatusToggleCompleted => {
            model.filters.status_completed = !model.filters.status_completed
        }
        Msg::FiltersStatusToggleWaiting => {
            model.filters.status_waiting = !model.filters.status_waiting
        }
        Msg::FiltersStatusToggleRecurring => {
            model.filters.status_recurring = !model.filters.status_recurring
        }
        Msg::FiltersPriorityToggleNone => {
            model.filters.priority_none = !model.filters.priority_none
        }
        Msg::FiltersPriorityToggleLow => model.filters.priority_low = !model.filters.priority_low,
        Msg::FiltersPriorityToggleMedium => {
            model.filters.priority_medium = !model.filters.priority_medium
        }
        Msg::FiltersPriorityToggleHigh => {
            model.filters.priority_high = !model.filters.priority_high
        }
        Msg::FiltersProjectChanged(new_project) => {
            let new_project = new_project.trim();
            model.filters.project = if new_project.is_empty() {
                Vec::new()
            } else {
                new_project
                    .split('.')
                    .map(std::borrow::ToOwned::to_owned)
                    .collect()
            }
        }
        Msg::FiltersTagsChanged(new_tags) => {
            let new_end = new_tags.ends_with(' ');
            model.filters.tags = if new_tags.is_empty() {
                Vec::new()
            } else {
                let mut tags: Vec<_> = new_tags
                    .split_whitespace()
                    .map(|s| s.trim().to_owned())
                    .collect();
                if new_end {
                    tags.push(String::new())
                }
                tags
            }
        }
        Msg::FiltersDescriptionChanged(new_description) => {
            model.filters.description_and_notes = new_description
        }
        Msg::FiltersReset => model.filters = Filters::default(),
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let selected_task = url.search().get(VIEW_TASK_SEARCH_KEY).and_then(|v| {
                uuid::Uuid::parse_str(v.first().unwrap_or(&String::new()))
                    .map(|uuid| {
                        if model.tasks.contains_key(&uuid) {
                            Some(uuid)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(None)
            });
            if selected_task.is_none() {
                url.set_search(UrlSearch::default()).go_and_replace();
            }
            model.selected_task = selected_task
        }
        Msg::EscapeKey => {
            if model.selected_task.is_some() {
                Urls::new(&model.base_url).home().go_and_push();
                model.selected_task = None
            }
        }
        Msg::ImportTasks => match window().prompt_with_message("Paste the tasks json here") {
            Ok(Some(content)) => {
                match serde_json::from_str::<HashMap<uuid::Uuid, Task>>(&content) {
                    Ok(tasks) => {
                        for (id, task) in tasks {
                            model.tasks.insert(id, task);
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
            let json = serde_json::to_string(&model.tasks);
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
        Msg::ViewTask(msg) => {pages::view_task::update(msg, model.page)}
    }
    LocalStorage::insert(TASKS_STORAGE_KEY, &model.tasks).expect("save tasks to LocalStorage");
    LocalStorage::insert(FILTERS_STORAGE_KEY, &model.filters)
        .expect("save filters to LocalStorage");
}

// ------ ------
//     View
// ------ ------

fn view(model: &Model) -> Node<Msg> {
    div![
        view_titlebar(),
        model.selected_task.map_or_else(
            || pages::home::view(model),
            |uuid| model.tasks.get(&uuid).map_or_else(
                || pages::home::view(model),
                |task| pages::view_task::view(model, task)
            )
        )
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
