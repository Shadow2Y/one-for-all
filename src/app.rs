use std::{
    io::{Error, Result},
    process::Output,
};

use crate::{
    config,
    engine::{self, discovery::format_cmd_type},
    models::{
        command::{Command, CommandKind, ExecutionMode::TemplateShell},
        request::ExecutionRequest,
    },
};
use clap::builder::styling::{AnsiColor, Effects};

pub fn handle(args: &[String]) -> Result<Output> {
    let (cmd, args) = args
        .split_first()
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidInput, "missing command"))?;
    if cmd == "eval" {
        return engine::execute_command(ExecutionRequest::new(
            cmd.to_owned(),
            Command {
                kind: TemplateShell,
                cmd: CommandKind::Args(args.to_vec()),
            },
            args.to_vec(),
        ));
    }
    Err(Error::new(
        std::io::ErrorKind::InvalidInput,
        "Invalid command",
    ))
}

use std::fmt::Write;

pub fn help() -> String {
    let cfg = config::get();

    let header = AnsiColor::BrightCyan.on_default() | Effects::BOLD;
    let command = AnsiColor::BrightGreen.on_default();

    let mut out = String::new();

    writeln!(out, "{header}Context:{header:#}").unwrap();

    match &cfg.profile {
        Some(profile) => {
            writeln!(out, "  {:10} {}", "Profile:", profile).unwrap();
        }
        None => {
            writeln!(out, "  {:10} <none>", "Profile:").unwrap();
        }
    }

    if let Some(local_path) = config::find_local_config() {
        writeln!(out, "  {:10} {}", "Config:", local_path.display()).unwrap();
    }

    writeln!(out, "\n{header}Commands:{header:#}").unwrap();

    let mut keys: Vec<&String> = cfg.commands.keys().collect();
    keys.sort();

    if keys.is_empty() {
        writeln!(out, "  (no commands configured)").unwrap();
    } else {
        for key in keys {
            if let Some(cmd) = cfg.commands.get(key) {
                writeln!(
                    out,
                    "  {command}{key}{command:#} {:>}",
                    format_cmd_type(cmd),
                )
                .unwrap();
            }
        }
    }

    writeln!(
        out,
        "  {command}app{command:#}       [builtin] -- application level commands"
    )
    .unwrap();

    out
}
