use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub run: String,

    #[serde(default)]
    pub params: Vec<String>,

    #[serde(default)]
    pub defaults: HashMap<String, String>,

    #[serde(default)]
    pub allow_trailing_args: bool,
}
