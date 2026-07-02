use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    run: String,
    params: Vec<String>,
    param_defaults: HashMap<String, String>,
}
