use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub run: String,

    #[serde(default = "empty_vec")]
    pub params: Vec<String>,

    #[serde(default = "empty_map")]
    pub param_defaults: HashMap<String, String>,
}

fn empty_map() -> HashMap<String, String> {
    HashMap::new()
}

fn empty_vec() -> Vec<String> {
    Vec::new()
}
