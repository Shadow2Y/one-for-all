use super::command::Command;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Variable {
    Provided(Provider),
    Literal(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provider {
    pub run: Command,
}
