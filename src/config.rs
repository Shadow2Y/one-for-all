use std::{collections::HashMap, fs::File, io::BufReader, sync::OnceLock};

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Config {
    _type: String,
    subtype: String,
    vars: HashMap<String, Value>,
    runnables: HashMap<String, Value>,
    lifecycle: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub struct GlobalConfig {
    vars: HashMap<String, Value>,
    runnables: HashMap<String, Value>,
    types: HashMap<String, HashMap<String, Config>>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    return CONFIG.get_or_init(|| load_config());
}

fn load_config() -> Config {
    let local_config = load_local_config();
    let global_config = load_global_config();

    return merge(&local_config, global_config);
}

fn merge(local: &Config, global: Config) -> Config {}

fn load_local_config() -> Config {
    let config_file = File::open(".ofa/config.json")
        .expect("Failed to open .ofa/config.json file. If not present, please use `ofa init`");
    return serde_json::from_reader(BufReader::new(config_file)).unwrap();
}

fn load_global_config() -> Config {}
