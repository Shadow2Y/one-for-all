use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use dynamic_function_macros::make_dyn;

use crate::{config, models::Value};

static TEMP_STORE: LazyLock<Mutex<HashMap<String, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[make_dyn]
pub fn set(key: String, value: Value) -> Value {
    TEMP_STORE.lock().unwrap().insert(key, value);
    Value::Void
}

#[make_dyn]
pub fn get(key: String) -> Value {
    log::debug!("Checking value for :: {}", key);

    // 1. Move the fallback into a reusable closure so we don't repeat ourselves
    let get_fallback = || {
        config::get()
            .get_vars(&key)
            .unwrap_or(&Value::Void)
            .to_owned()
    };

    // 2. Safely acquire the lock
    match TEMP_STORE.lock() {
        Ok(val) => {
            // 3. Check if the key actually exists in the map
            match val.get(&key) {
                Some(stored_value) => stored_value.to_owned(),
                None => get_fallback(), // Key missing? Go to fallback.
            }
        }
        Err(_) => get_fallback(), // Lock poisoned? Go to fallback.
    }
}
