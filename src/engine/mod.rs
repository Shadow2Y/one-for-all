use anyhow::Result;

use crate::{
    config, context,
    models::{
        Value,
        command::{Command, CommandKind},
    },
};

mod executor;
mod resolver;
pub mod tokenizer;

pub use executor::execute_command;


// ── CLI entry point ───────────────────────────────────────────────────────────

/// Resolves the CLI command string (possibly a subcommand path) and executes
/// the matched leaf command.
pub fn handle_command(cmd: &str, args: &[String]) -> Result<Value> {
    let (leaf, remaining_args) = find_leaf(cmd, args)?;
    execute_command(context::get_registry(), leaf, remaining_args)
}

// ── Command resolution ────────────────────────────────────────────────────────

/// Walks the command tree for `cmd`, consuming leading `args` entries that
/// match group subcommand names until a leaf (non-group) command is reached.
///
/// Returns a reference to the leaf [`Command`] and the unconsumed args slice.
fn find_leaf<'a>(cmd: &str, args: &'a [String]) -> Result<(&'static Command, &'a [String])> {
    let config = config::get();

    let mut current = config
        .commands
        .get(cmd)
        .ok_or_else(|| anyhow::anyhow!("Unknown command '{cmd}'"))?;

    let mut consumed = 0;

    while let CommandKind::Group(children) = &current.cmd {
        match args.get(consumed).and_then(|a| children.get(a)) {
            Some(next) => {
                current = next;
                consumed += 1;
            }
            // No matching subcommand — stop here (caller may want the group's
            // default behaviour or will surface an error via execute_command).
            None => break,
        }
    }

    Ok((current, &args[consumed..]))
}
