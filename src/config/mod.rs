use config::Config;
use std::fs;
use std::sync::OnceLock;

mod config;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get() -> &'static Config {
    CONFIG.get_or_init(|| {
        let content =
            fs::read_to_string(path()).expect("Critical Error: Could not read config file.");
        toml::from_str(&content).expect("unable to deser")
    })
}

fn path() -> String {
    let home = std::env::var("HOME").expect("HOME not set");
    format!("{home}/.ofa/global.toml")
}
