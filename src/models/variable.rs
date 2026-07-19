use crate::models::Value;

use super::command::Command;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Variable {
    Provided(Provider),
    Literal(Value),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provider {
    pub run: Command,

    #[serde(default)]
    pub cache: Cache,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Cache {
    pub ttl: Option<u64>,
    pub persistent: bool,
    pub namespace: Option<String>,
    pub signature: Option<Command>,
}
