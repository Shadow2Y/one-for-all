use std::{io::Result, process::Output};

use crate::models::request::ExecutionRequest;

pub mod discovery;
mod executor;
mod resolver;
pub mod tokenizer;
pub use executor::execute_command;

// ── CLI entry point ───────────────────────────────────────────────────────────

/// Resolves the CLI command string (possibly a subcommand path) and executes
/// the matched leaf command, or returns a subcommand listing if a command group is targeted.
pub fn run(request: ExecutionRequest) -> Result<Output> {
    execute_command(request)
}
