use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::OnceLock;

// ── Runnable ──────────────────────────────────────────────────────────────────
//
// A runnable is the core unit of work in ofa. Three forms are supported:
//
//   "command": "sh string with #var interpolation"   → Shell
//   "command": ["argv", "array"]                     → Argv
//   "command": { "args": ["a","b"], "command": "…" } → Function (named args)

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Runnable {
    /// Inline shell string. #var references are interpolated before execution.
    Shell(String),
    /// Direct argv array — passed straight to execvp, no shell involved.
    Argv(Vec<String>),
    /// Named-parameter function with an explicit arg list and a shell command body.
    Function { args: Vec<String>, command: String },
}

// ── Local config (.ofa/config.json) ──────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LocalConfig {
    #[serde(rename = "type")]
    pub project_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, Value>,
    #[serde(default)]
    pub runnables: HashMap<String, Runnable>,
    #[serde(default)]
    pub commands: HashMap<String, Runnable>,
    #[serde(default)]
    pub lifecycle: HashMap<String, Runnable>,
}

// ── Global config (~/.ofa/config.json) ───────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypeDefault {
    #[serde(default)]
    pub vars: HashMap<String, Value>,
    #[serde(default)]
    pub runnables: HashMap<String, Runnable>,
    #[serde(default)]
    pub commands: HashMap<String, Runnable>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypeConfig {
    #[serde(default)]
    pub vars: HashMap<String, Value>,
    #[serde(default)]
    pub runnables: HashMap<String, Runnable>,
    /// Default settings applied to all projects of this type.
    pub default: Option<TypeDefault>,
    /// Named sub-variants (e.g. "maven" inside "java").
    pub subtypes: Option<HashMap<String, TypeDefault>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub vars: HashMap<String, Value>,
    #[serde(default)]
    pub runnables: HashMap<String, Runnable>,
    #[serde(default)]
    pub types: HashMap<String, TypeConfig>,
}

// ── Resolved (merged) config ──────────────────────────────────────────────────
//
// This is what the engine actually uses — the result of layering global defaults,
// type config, subtype config, and local config on top of each other.

#[derive(Debug, Clone, Default)]
pub struct ResolvedConfig {
    pub vars: HashMap<String, Value>,
    /// Named commands exposed to the user (e.g. `ofa build`).
    pub commands: HashMap<String, Runnable>,
    /// Internal runnables used for interpolation / lifecycle hooks.
    pub runnables: HashMap<String, Runnable>,
    /// Lifecycle hooks (pre-command, post-command, etc.).
    pub lifecycle: HashMap<String, Runnable>,
}

// ── Static cache ──────────────────────────────────────────────────────────────

static CONFIG: OnceLock<ResolvedConfig> = OnceLock::new();

/// Try to load and cache the config. Returns `None` gracefully if no config
/// exists (e.g. when displaying help outside an ofa project).
pub fn try_get_config() -> Option<&'static ResolvedConfig> {
    if CONFIG.get().is_none() {
        if let Ok(cfg) = load_config() {
            let _ = CONFIG.set(cfg);
        }
    }
    CONFIG.get()
}

/// Load and cache the config, returning an error if no local config is found.
pub fn get_config() -> Result<&'static ResolvedConfig> {
    if CONFIG.get().is_none() {
        let cfg = load_config()?;
        let _ = CONFIG.set(cfg);
    }
    Ok(CONFIG.get().expect("config must be set after load_config succeeds"))
}

// ── Loading ───────────────────────────────────────────────────────────────────

fn load_config() -> Result<ResolvedConfig> {
    let local = load_local_config()?;
    // Global config is optional — fall back to empty defaults if missing.
    let global = load_global_config().unwrap_or_default();
    Ok(merge(local, global))
}

fn load_local_config() -> Result<LocalConfig> {
    load_json_file(".ofa/config.json")
        .context("No local config found. Run `ofa init` to set up this project.")
}

fn load_global_config() -> Option<GlobalConfig> {
    let path = global_config_path()?;
    load_json_file::<GlobalConfig>(path.to_str()?).ok()
}

fn global_config_path() -> Option<PathBuf> {
    // Use $HOME — tilde (~) is NOT expanded by std::fs::File::open.
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".ofa/config.json"))
}

fn load_json_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = File::open(path).context(format!("Cannot open '{}'", path))?;
    serde_json::from_reader(BufReader::new(file))
        .context(format!("Failed to parse JSON in '{}'", path))
}

// ── Merge ─────────────────────────────────────────────────────────────────────
//
// Priority (lowest → highest):
//   global base vars/runnables
//     → type-level vars/runnables
//       → type default (base subtype) vars/runnables/commands
//         → named subtype vars/runnables/commands
//           → local vars/runnables/commands (highest priority)

fn merge(local: LocalConfig, global: GlobalConfig) -> ResolvedConfig {
    let mut vars = global.vars;
    let mut runnables = global.runnables;
    let mut commands = HashMap::new();
    let lifecycle = local.lifecycle.clone();

    if let Some(type_name) = &local.project_type {
        if let Some(type_cfg) = global.types.get(type_name.as_str()) {
            vars.extend(type_cfg.vars.clone());
            runnables.extend(type_cfg.runnables.clone());

            if let Some(default) = &type_cfg.default {
                vars.extend(default.vars.clone());
                runnables.extend(default.runnables.clone());
                commands.extend(default.commands.clone());
            }

            if let Some(subtype) = &local.subtype {
                if let Some(subtypes) = &type_cfg.subtypes {
                    if let Some(sub) = subtypes.get(subtype.as_str()) {
                        vars.extend(sub.vars.clone());
                        runnables.extend(sub.runnables.clone());
                        commands.extend(sub.commands.clone());
                    }
                }
            }
        }
    }

    // Local always wins.
    vars.extend(local.vars);
    runnables.extend(local.runnables);
    commands.extend(local.commands);

    ResolvedConfig { vars, commands, runnables, lifecycle }
}
