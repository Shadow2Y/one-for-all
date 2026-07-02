use super::function::Function;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Command {
    Script(String),
    Args(Vec<String>),
    Group(HashMap<String, Command>),
    Parameterized(Function),
}
