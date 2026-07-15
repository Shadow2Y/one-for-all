use serde::{Deserialize, Serialize};

use crate::models::{Value, command::Command, provider::Provider};

use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "empty_map")]
    pub env: HashMap<String, String>,

    #[serde(default = "empty_vars")]
    pub vars: HashMap<String, Value>,

    #[serde(default = "empty_commands")]
    pub commands: HashMap<String, Command>,

    #[serde(default = "empty_providers")]
    pub providers: HashMap<String, Provider>,
}

impl Config {
    pub fn get_vars(&self, key: &String) -> Option<&Value> {
        self.vars.get(key)
    }
}

fn empty_map() -> HashMap<String, String> {
    HashMap::new()
}

fn empty_vars() -> HashMap<String, Value> {
    HashMap::new()
}

fn empty_commands() -> HashMap<String, Command> {
    HashMap::new()
}

fn empty_providers() -> HashMap<String, Provider> {
    HashMap::new()
}
