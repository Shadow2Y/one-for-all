use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ExecutionContext {
    variables: RwLock<HashMap<String, String>>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_var(&self, name: impl Into<String>, value: impl Into<String>) {
        self.variables
            .write()
            .unwrap()
            .insert(name.into(), value.into());
    }

    pub fn get_var(&self, name: &str) -> Result<String> {
        self.variables
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Variable '{}' not found", name))
    }
}
