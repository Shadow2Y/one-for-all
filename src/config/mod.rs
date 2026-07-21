pub use config::Config;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

mod config;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get() -> &'static Config {
    CONFIG.get_or_init(load_merged_config)
}

/// Returns system config directory: `$XDG_CONFIG_HOME/ofa` or `~/.config/ofa`
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not determine system config directory")
        .join("ofa")
}

pub fn global_config_path() -> PathBuf {
    config_dir().join("global.toml")
}

pub fn store_path() -> PathBuf {
    config_dir().join("store")
}

pub fn profiles_dir() -> PathBuf {
    config_dir().join("profiles")
}

/// Loads and resolves global config, profile config, and local repo config in order:
/// 1. Global config (~/.config/ofa/global.toml)
/// 2. Profile config (~/.config/ofa/profiles/<name>.toml) if specified
/// 3. Local repo config (.ofa.toml or ofa.toml)
fn load_merged_config() -> Config {
    let mut resolved = load_file(&global_config_path()).unwrap_or_default();

    let local_config = find_local_config().and_then(|path| load_file(&path));

    // Profile specified in local config takes precedence, fallback to global profile if any
    let profile_name = local_config
        .as_ref()
        .and_then(|c| c.profile.clone())
        .or_else(|| resolved.profile.clone());

    if let Some(ref name) = profile_name {
        if let Some(profile_cfg) = load_profile(name) {
            resolved.merge(profile_cfg);
        } else {
            log::warn!("Profile '{}' specified, but profile file not found", name);
        }
    }

    if let Some(local) = local_config {
        resolved.merge(local);
    }

    resolved
}

/// Tries to load a profile from `~/.config/ofa/profiles/<name>.toml` or `~/.config/ofa/store/profiles/<name>.toml`
pub fn load_profile(name: &str) -> Option<Config> {
    let filename = format!("{name}.toml");
    let primary = profiles_dir().join(&filename);
    if primary.exists() {
        return load_file(&primary);
    }
    let secondary = store_path().join("profiles").join(&filename);
    if secondary.exists() {
        return load_file(&secondary);
    }
    None
}

fn load_file(path: &Path) -> Option<Config> {
    let content = fs::read_to_string(path).ok()?;
    match toml::from_str(&content) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            log::error!("Failed to parse config file {:?}: {}", path, e);
            None
        }
    }
}

/// Searches upwards from current directory for `.ofa.toml` or `ofa.toml`
pub fn find_local_config() -> Option<PathBuf> {
    let mut curr = env::current_dir().ok()?;
    loop {
        let dot_ofa = curr.join(".ofa.toml");
        if dot_ofa.is_file() {
            return Some(dot_ofa);
        }
        let ofa = curr.join("ofa.toml");
        if ofa.is_file() {
            return Some(ofa);
        }
        if !curr.pop() {
            break;
        }
    }
    None
}

