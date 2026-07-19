use super::function::Function;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    #[serde(rename = "type")]
    pub kind: ExecutionMode,

    pub cmd: CommandKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Shell,
    Template,
    TemplateShell,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommandKind {
    Script(String),
    Args(Vec<String>),
    Parameterized(Function),
    Group(HashMap<String, Command>),
}
