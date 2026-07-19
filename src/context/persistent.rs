use std::{collections::HashMap, fs};

use anyhow::Result;

use crate::{config, models::Value};

pub trait StoreProvider {
    fn get(&self, namespace: &str, key: &str) -> Option<Value>;

    fn set(&mut self, namespace: &str, key: String, value: Value) -> Result<()>;
}

pub struct TomlStore;

impl TomlStore {
    pub fn new() -> Self {
        Self
    }
}

impl StoreProvider for TomlStore {
    fn get(&self, namespace: &str, key: &str) -> Option<Value> {
        let path = config::store_path();

        let content = fs::read_to_string(path).ok()?;

        let store: HashMap<String, HashMap<String, Value>> = toml::from_str(&content).ok()?;

        store.get(namespace).and_then(|ns| ns.get(key)).cloned()
    }

    fn set(&mut self, namespace: &str, key: String, value: Value) -> Result<()> {
        let path = config::store_path();

        let mut store: HashMap<String, HashMap<String, Value>> = fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();

        store
            .entry(namespace.to_string())
            .or_default()
            .insert(key, value);

        fs::write(path, toml::to_string_pretty(&store)?)?;

        Ok(())
    }
}
