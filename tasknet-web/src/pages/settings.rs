use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    fmt::Display,
};

#[allow(clippy::wildcard_imports)]
use seed::{prelude::*, *};

use crate::{
    components::{duration_string, view_button, view_checkbox, view_text_input},
    settings::Settings,
    task::{Id, Priority, Status, Task},
    urgency, Filters, GlobalModel, Msg as GMsg,
};

const SETTINGS_STORAGE_KEY: &str = "tasknet-filters";

pub fn init() -> Model {
    let settings = match LocalStorage::get(SETTINGS_STORAGE_KEY) {
        Ok(settings) => settings,
        Err(seed::browser::web_storage::WebStorageError::SerdeError(err)) => {
            panic!("failed to parse settings: {:?}", err)
        }
        Err(_) => Settings::default(),
    };
    Model { settings }
}

#[derive(Debug)]
pub struct Model {
    settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Msg {}

#[allow(clippy::too_many_lines)]
pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<GMsg>) {
    match msg {}
    LocalStorage::insert(SETTINGS_STORAGE_KEY, &model.settings)
        .expect("save settings to LocalStorage");
}

pub fn view(global_model: &GlobalModel, model: &Model) -> Node<GMsg> {
    div!["Settings"]
}
