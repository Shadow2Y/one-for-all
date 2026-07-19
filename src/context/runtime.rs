use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use crate::models::Value;

static TEMP_STORE: LazyLock<Mutex<HashMap<String, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn set(key: String, value: Value) {
    TEMP_STORE.lock().unwrap().insert(key, value);
}

pub fn get(key: &str) -> Option<Value> {
    TEMP_STORE.lock().ok().and_then(|map| map.get(key).cloned())
}
