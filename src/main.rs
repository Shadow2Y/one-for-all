use anyhow::{Result, bail};
use clap::Parser;

use crate::models::Value;

mod app;
mod config;
mod context;
mod engine;
mod models;

#[derive(Parser)]
#[command(
    name = "ofa",
    about = "The one CLI tool to orchestrate them all",
    version,
    arg_required_else_help = false
)]
struct CLI {
    /// The command to run (built-in or from project config).
    command: Option<String>,

    /// Additional arguments forwarded to the command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let cli = CLI::parse();

    let result: Result<Value> = match cli.command.as_deref() {
        Some("app") => app::handle(&cli.args),
        Some(cmd) => engine::handle_command(cmd, &cli.args),
        None => app::help(),
    };

    match result {
        Ok(val) => {
            // Handle and format the output dynamically for the CLI user
            if let Err(e) = handle_cli_output(val) {
                log::error!("{:#}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            log::error!("{:#}", e);
            std::process::exit(1);
        }
    }
}

fn handle_cli_output(val: Value) -> Result<()> {
    match val {
        Value::Object(mut map) => {
            // Check if this object looks like a shell command execution result
            if map.contains_key("stdout") && map.contains_key("exit_code") {
                let exit_code = match map.remove("exit_code") {
                    Some(Value::Int(code)) => code,
                    _ => 0,
                };

                let stdout = match map.remove("stdout") {
                    Some(Value::String(s)) => s,
                    Some(other) => format!("{}", other),
                    None => String::new(),
                };

                let stderr = match map.remove("stderr") {
                    Some(Value::String(s)) => s,
                    Some(other) => format!("{}", other),
                    None => String::new(),
                };

                // 1. If the script failed, bubble it up as a true CLI application error
                if exit_code != 0 {
                    if !stderr.trim().is_empty() {
                        bail!(
                            "Script failed (exit status {}):\n{}",
                            exit_code,
                            stderr.trim()
                        );
                    } else {
                        bail!("Script failed with exit status {}", exit_code);
                    }
                }

                // 2. If it passed, print clean stdout.
                // We use print! instead of println! because stdout usually ends with its own \n
                print!("{}", stdout);
                Ok(())
            } else {
                // It's a normal map/object from your engine, print it normally
                println!("{:?}", Value::Object(map));
                Ok(())
            }
        }
        // Handle plain strings cleanly (without quotes)
        Value::String(s) => {
            println!("{}", s);
            Ok(())
        }
        // Don't print anything for Void
        Value::Void => Ok(()),
        // Fallback for numbers, booleans, etc.
        other => {
            println!("{:?}", other);
            Ok(())
        }
    }
}
