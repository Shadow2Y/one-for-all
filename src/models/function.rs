use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    pub run: String,

    #[serde(default = "empty_vec")]
    pub params: Vec<String>,
}

fn empty_vec() -> Vec<String> {
    Vec::new()
}
