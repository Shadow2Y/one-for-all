use std::env;

use dynamic_function_macros::make_dyn;

use crate::{config, context::store, models::Value};

// ── Arithmetic ────────────────────────────────────────────────────────────────

#[make_dyn]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[make_dyn]
pub fn sub(a: u32, b: u32) -> u32 {
    a - b
}

#[make_dyn]
pub fn mul(a: u32, b: u32) -> u32 {
    a * b
}

#[make_dyn]
pub fn div(a: u32, b: u32) -> u32 {
    a / b
}

#[make_dyn]
pub fn modulus(a: u32, b: u32) -> u32 {
    a % b
}

// ── Comparison ────────────────────────────────────────────────────────────────

#[make_dyn]
pub fn equals(arg0: Value, arg1: Value) -> bool {
    arg0 == arg1
}

// ── Environment ───────────────────────────────────────────────────────────────

/// Looks up a key in the config `[env]` table first, then falls back to the
/// real process environment.
#[make_dyn]
pub fn env(key: String) -> String {
    config::get()
        .env
        .get(&key)
        .unwrap_or(&env::var(&key).unwrap_or_default())
        .clone()
}

// ── Control / side-effects ────────────────────────────────────────────────────

pub fn println(args: &[Value]) -> Result<Value, String> {
    println!("{:?}", args);
    Ok(Value::Void)
}

pub fn throw(args: &[Value]) -> Result<Value, String> {
    Err(format!("{:?}", args))
}

pub fn suppress(args: &[Value]) -> Result<Value, String> {
    log::debug!("Suppressing args :: {:?}", args);
    Ok(Value::Void)
}

// ── Variable store ────────────────────────────────────────────────────────────

/// Sets a key in the *session* (thread-local) store only.
#[make_dyn]
pub fn set(key: String, value: Value) -> Value {
    store::set(key, value);
    Value::Void
}

/// Fetches a key from the session store, then falls back to the persistent
/// store. Panics if the key is not found (matches original behaviour).
#[make_dyn]
pub fn get(key: String) -> Value {
    store::fetch(&key).unwrap_or_else(|| panic!("Value not found for key :: {key}"))
}

/// Writes a key to both the session store and the persistent (TOML) store.
#[make_dyn]
pub fn store(key: String, value: Value) -> Value {
    store::persist(key, value);
    Value::Void
}
