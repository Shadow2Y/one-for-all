pub use config::Config;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

mod config;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get() -> &'static Config {
    CONFIG.get_or_init(|| {
        let content =
            fs::read_to_string(config_path()).expect("Critical Error: Could not read config file.");
        toml::from_str(&content).expect("Critical Error: Invalid config file.")
    })
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ofa")
        .join("global.toml")
}

pub fn store_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".ofa")
        .join("store")
}
