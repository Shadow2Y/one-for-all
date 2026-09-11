use std::{
    io::Error,
    process::{ExitStatus, Output},
};

use crate::{
    config,
    models::command::{Command, CommandKind},
};

pub mod discovery;
mod executor;
mod resolver;
pub mod tokenizer;

pub use discovery::list_group_subcommands;
pub use executor::execute_command;

// ── CLI entry point ───────────────────────────────────────────────────────────

/// Resolves the CLI command string (possibly a subcommand path) and executes
/// the matched leaf command, or returns a subcommand listing if a command group is targeted.
pub fn handle_command(cmd: &str, args: &[String]) -> Result<Output, Error> {
    let (leaf, remaining_args, path) = find_leaf(cmd, args)?;

    match &leaf.cmd {
        CommandKind::Group(children) => {
            let text = list_group_subcommands(&path, children);

            Ok(Output {
                status: ExitStatus::default(),
                stdout: text.into_bytes(),
                stderr: Vec::new(),
            })
        }

        _ => execute_command(leaf, remaining_args),
    }
}

// ── Command resolution ────────────────────────────────────────────────────────

/// Walks the command tree for `cmd`, consuming leading `args` entries that
/// match group subcommand names until a leaf (non-group) command is reached.
///
/// Returns a reference to the leaf [`Command`], the unconsumed args slice, and the resolved path.
fn find_leaf<'a>(
    cmd: &str,
    args: &'a [String],
) -> Result<(&'static Command, &'a [String], String), Error> {
    let config = config::get();

    let mut current = config.commands.get(cmd).ok_or_else(|| {
        Error::new(
            std::io::ErrorKind::NotFound,
            format!("Unknown command '{cmd}'"),
        )
    })?;

    let mut consumed = 0;
    let mut path = cmd.to_string();

    while let CommandKind::Group(children) = &current.cmd {
        match args.get(consumed).and_then(|a| children.get(a)) {
            Some(next) => {
                path.push(' ');
                path.push_str(args[consumed].as_str());
                current = next;
                consumed += 1;
            }
            None => break,
        }
    }

    Ok((current, &args[consumed..], path))
}
