use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    #[serde(rename = "type")]
    pub kind: ExecutionMode,

    pub cmd: String,

    #[serde(default)]
    pub params: Vec<String>,

    #[serde(default)]
    pub defaults: HashMap<String, String>,

    #[serde(default)]
    pub allow_trailing_args: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Shell,
    TemplateShell,
}
