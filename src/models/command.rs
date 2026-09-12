use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    #[serde(rename = "type", default = "kind")]
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

impl Command {
    pub fn new(kind: ExecutionMode, cmd: String) -> Self {
        Command {
            kind: kind,
            cmd: cmd,
            params: Vec::new(),
            defaults: HashMap::new(),
            allow_trailing_args: false,
        }
    }
}

fn kind() -> ExecutionMode {
    ExecutionMode::TemplateShell
}
