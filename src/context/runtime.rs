use std::{cell::RefCell, collections::HashMap};

use crate::models::Value;

// ── Thread-local session store ────────────────────────────────────────────────
//
// Using `thread_local!` instead of a global `Mutex<HashMap>` gives us free
// execution isolation: each OS thread (and therefore each concurrent command
// invocation) operates on its own independent variable map.
//
// For future async support, replace with `tokio::task_local!` or thread the
// vars through an `Arc<ExecutionContext>` in the resolver call-chain — the
// resolver API (`local_vars: &HashMap`) is already shaped for that transition.

thread_local! {
    static VARS: RefCell<HashMap<String, Value>> = RefCell::new(HashMap::new());
}

pub fn set(key: String, value: Value) {
    VARS.with(|store| store.borrow_mut().insert(key, value));
}

pub fn get(key: &str) -> Option<Value> {
    VARS.with(|store| store.borrow().get(key).cloned())
}
