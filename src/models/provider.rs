use super::command::Command;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Provider {
    #[serde(default = "provider_store")]
    store: bool,
    global: bool,
    fetch: Command,
    ttl: Option<u64>,
    signature: Option<Command>,
}

fn provider_store() -> bool {
    return true;
}
