use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{command::Command, variable::Variable};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub vars: HashMap<String, Variable>,

    #[serde(default)]
    pub commands: HashMap<String, Command>,
}
