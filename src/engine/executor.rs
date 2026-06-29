use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::config::Runnable;
use super::interpolator::interpolate;

// ── User-facing execution (streams stdout/stderr live) ────────────────────────

/// Run a runnable as a top-level user command, streaming output directly to the
/// terminal. `user_args` are bound positionally to the function's declared arg
/// names (for `Runnable::Function`).
pub fn execute_user(
    runnable: &Runnable,
    user_args: &[String],
    vars: &HashMap<String, Value>,
) -> Result<()> {
    let status = match runnable {
        Runnable::Shell(cmd) => {
            let cmd = interpolate(cmd, vars);
            log::debug!("exec shell: {}", cmd);
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .status()
                .context("Failed to spawn shell")?
        }

        Runnable::Argv(argv) => {
            let argv: Vec<String> = argv.iter().map(|a| interpolate(a, vars)).collect();
            anyhow::ensure!(!argv.is_empty(), "Runnable argv array is empty");
            log::debug!("exec argv: {:?}", argv);
            Command::new(&argv[0])
                .args(&argv[1..])
                .status()
                .context(format!("Failed to spawn '{}'", argv[0]))?
        }

        Runnable::Function { args: fn_args, command } => {
            let local_vars = bind_args(fn_args, user_args, vars);
            let cmd = interpolate(command, &local_vars);
            log::debug!("exec fn shell: {}", cmd);
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .status()
                .context("Failed to spawn shell for function runnable")?
        }
    };

    anyhow::ensure!(
        status.success(),
        "Command exited with code {}",
        status.code().unwrap_or(-1)
    );
    Ok(())
}

// ── Captured execution (returns stdout as String) ─────────────────────────────

/// Run a runnable and capture its stdout — used when a runnable is referenced
/// inside a `$(...)` call expression. Stderr is still forwarded to the terminal.
pub fn execute_capture(
    runnable: &Runnable,
    user_args: &[String],
    vars: &HashMap<String, Value>,
) -> Result<String> {
    let output = match runnable {
        Runnable::Shell(cmd) => {
            let cmd = interpolate(cmd, vars);
            log::debug!("capture shell: {}", cmd);
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()
                .context("Failed to spawn shell")?
        }

        Runnable::Argv(argv) => {
            let argv: Vec<String> = argv.iter().map(|a| interpolate(a, vars)).collect();
            anyhow::ensure!(!argv.is_empty(), "Runnable argv array is empty");
            Command::new(&argv[0])
                .args(&argv[1..])
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()
                .context(format!("Failed to spawn '{}'", argv[0]))?
        }

        Runnable::Function { args: fn_args, command } => {
            let local_vars = bind_args(fn_args, user_args, vars);
            let cmd = interpolate(command, &local_vars);
            log::debug!("capture fn shell: {}", cmd);
            Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()
                .context("Failed to spawn shell")?
        }
    };

    anyhow::ensure!(
        output.status.success(),
        "Captured command exited with code {}",
        output.status.code().unwrap_or(-1)
    );

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Bind positional `user_args` to a function's declared `fn_args` names,
/// layering them on top of the existing `vars` map.
fn bind_args(
    fn_args: &[String],
    user_args: &[String],
    vars: &HashMap<String, Value>,
) -> HashMap<String, Value> {
    let mut local = vars.clone();
    for (name, value) in fn_args.iter().zip(user_args.iter()) {
        local.insert(name.clone(), Value::String(value.clone()));
    }
    local
}
