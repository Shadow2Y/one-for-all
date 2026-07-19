use super::persistent::StoreProvider;
use crate::{
    context::{self, persistent::TomlStore, runtime},
    models::Value,
};
use std::sync::{LazyLock, Mutex};

static STORE: LazyLock<Mutex<TomlStore>> = LazyLock::new(|| Mutex::new(TomlStore::new()));

// ── Session (thread-local) store ──────────────────────────────────────────────

/// Writes `value` to the current thread's session store only.
/// Does not survive the process.
pub fn set(key: String, value: Value) {
    runtime::set(key, value);
}

// ── Persistent store ──────────────────────────────────────────────────────────

/// Writes `value` to both the session store and the on-disk TOML store.
pub fn persist(key: String, value: Value) {
    runtime::set(key.clone(), value.clone());
    let ns = context::current();
    STORE.lock().unwrap().set(&ns, key, value).unwrap();
}

// ── Unified fetch ─────────────────────────────────────────────────────────────

/// Lookup order: thread-local session → persistent namespace → global namespace.
pub fn fetch(key: &str) -> Option<Value> {
    if let Some(value) = runtime::get(key) {
        return Some(value);
    }

    let store = STORE.lock().unwrap();
    let ns = context::current();
    store.get(&ns, key).or_else(|| store.get("global", key))
}
