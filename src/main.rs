use std::any::Any;

use anyhow::Result;
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
        Ok(val) => println!("{val}"),
        Err(e) => {
            log::error!("{:#}", e);
            std::process::exit(1);
        }
    }
}
