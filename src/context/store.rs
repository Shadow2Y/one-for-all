use std::sync::{LazyLock, Mutex};

use super::persistent::StoreProvider;
use crate::{
    context::{self, persistent::TomlStore, runtime},
    models::Value,
};

static STORE: LazyLock<Mutex<TomlStore>> = LazyLock::new(|| Mutex::new(TomlStore::new()));

pub fn set(key: String, value: Value) {
    runtime::set(key.clone(), value.clone());
}

pub fn store(key: String, value: Value) {
    runtime::set(key.clone(), value.clone());

    let ns = context::current();
    STORE.lock().unwrap().set(&ns, key, value).unwrap();
}

pub fn fetch(key: &str) -> Option<Value> {
    if let Some(value) = runtime::get(key) {
        return Some(value);
    }

    let store = STORE.lock().unwrap();
    let ns = context::current();

    store.get(&ns, key).or_else(|| store.get("global", key))
}
