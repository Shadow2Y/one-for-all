use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use dynamic_function_macros::make_dyn;

use crate::models::Value;

static DATA_PATH: &str = "~/.ofa/store";
static TEMP_STORE: LazyLock<Mutex<HashMap<String, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[make_dyn]
pub fn set(key: String, value: Value) -> Value {
    TEMP_STORE.lock().unwrap().insert(key, value);
    Value::Void
}

#[make_dyn]
pub fn get(key: String) -> Value {
    TEMP_STORE.lock().unwrap().get(&key).unwrap().to_owned()
}
