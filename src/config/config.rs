use serde::{Deserialize, Serialize};

use crate::models::{command::Command, provider::Provider};

use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    env: HashMap<String, String>,
    vars: HashMap<String, String>,
    commands: HashMap<String, Command>,
    providers: HashMap<String, Provider>,
}
