use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::LocalConfig;

/// The default global config, embedded at compile time so `ofa init --global`
/// works without needing to bundle the defaults/ folder alongside the binary.
const DEFAULT_GLOBAL_CONFIG: &str = include_str!("../defaults/global.json");

pub fn handle_init(global: bool) -> Result<()> {
    if global {
        init_global()
    } else {
        init_local()
    }
}

// ── Local init (.ofa/) ────────────────────────────────────────────────────────

fn init_local() -> Result<()> {
    let config_dir = Path::new(".ofa");
    let data_dir = config_dir.join("data");
    let config_path = config_dir.join("config.json");

    // Create directory structure.
    fs::create_dir_all(&data_dir).context("Failed to create .ofa/data/")?;

    if config_path.exists() {
        log::warn!(".ofa/config.json already exists — skipping creation.");
        println!("⚠  .ofa/config.json already exists. Delete it first if you want to re-init.");
        return Ok(());
    }

    let project_type = detect_project_type();
    let type_str = project_type.unwrap_or("unknown");

    let config = LocalConfig {
        project_type: Some(type_str.to_string()),
        subtype: None,
        vars: HashMap::new(),
        runnables: HashMap::new(),
        commands: HashMap::new(),
        lifecycle: HashMap::new(),
    };

    let json = serde_json::to_string_pretty(&config).context("Failed to serialize config")?;
    fs::write(&config_path, json).context("Failed to write .ofa/config.json")?;

    println!("✓  Initialized ofa project (type: {})", type_str);
    println!("   Edit .ofa/config.json to add commands, vars, and runnables.");
    log::info!("Initialized ofa at .ofa/config.json with type={}", type_str);

    Ok(())
}

// ── Global init (~/.ofa/) ─────────────────────────────────────────────────────

fn init_global() -> Result<()> {
    let home = std::env::var("HOME").context("$HOME is not set")?;
    let global_dir = Path::new(&home).join(".ofa");
    let data_dir = global_dir.join("data");
    let config_path = global_dir.join("config.json");

    fs::create_dir_all(&data_dir).context("Failed to create ~/.ofa/data/")?;

    if config_path.exists() {
        log::warn!("~/.ofa/config.json already exists — skipping.");
        println!("⚠  ~/.ofa/config.json already exists. Delete it first to re-init.");
        return Ok(());
    }

    fs::write(&config_path, DEFAULT_GLOBAL_CONFIG)
        .context("Failed to write ~/.ofa/config.json")?;

    println!("✓  Initialized global ofa config at ~/.ofa/config.json");
    log::info!("Global ofa config written to {}", config_path.display());

    Ok(())
}

// ── Project-type detection ────────────────────────────────────────────────────

fn detect_project_type() -> Option<&'static str> {
    const INDICATORS: &[(&str, &str)] = &[
        ("pom.xml", "java"),
        ("build.gradle", "java"),
        ("build.gradle.kts", "java"),
        ("Cargo.toml", "rust"),
        ("package.json", "node"),
        ("go.mod", "go"),
        ("pyproject.toml", "python"),
        ("setup.py", "python"),
        ("Makefile", "make"),
        ("CMakeLists.txt", "cmake"),
        ("composer.json", "php"),
        ("Gemfile", "ruby"),
    ];

    for (file, lang) in INDICATORS {
        if Path::new(file).exists() {
            log::info!("Detected project type '{}' via {}", lang, file);
            return Some(lang);
        }
    }
    None
}
