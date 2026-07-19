use serde::{Deserialize, Serialize};

use crate::models::{
    Value,
    command::Command,
    variable::{Provider, Variable},
};

use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "empty_map")]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub vars: HashMap<String, Variable>,

    #[serde(default = "empty_commands")]
    pub commands: HashMap<String, Command>,
}

impl Config {
    pub fn get_vars(&self, key: &String) -> Option<&Variable> {
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
