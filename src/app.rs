use std::{
    io::{Error, Result},
    process::Output,
};

use crate::{
    config,
    engine::{self},
    models::{
        command::{
            Command,
            ExecutionMode::{self, TemplateShell},
        },
        request::ExecutionRequest,
    },
};
use clap::builder::styling::{Effects, Style};

pub fn handle(args: &[String]) -> Result<Output> {
    let (cmd, args) = args
        .split_first()
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidInput, "missing command"))?;
    if cmd == "eval" {
        return engine::execute_command(ExecutionRequest::new(
            Command::new(TemplateShell, args.join(" ")),
            Vec::new(),
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

    let mut out = String::new();

    let header = Style::new().effects(Effects::BOLD | Effects::UNDERLINE);
    let command = Style::new().effects(Effects::BOLD);

    writeln!(out, "{header}Context:{header:#}").unwrap();

    match &cfg.profile {
        Some(profile) => {
            writeln!(out, "  {command}{:10}{command:#} \t{}", "Profile:", profile).unwrap();
        }
        None => {
            writeln!(out, "  {command}{:10}{command:#} \t<none>", "Profile:").unwrap();
        }
    }

    writeln!(
        out,
        "  {command}{:10}{command:#} \t{:?}",
        "Config dir:",
        config::config_dir()
    )
    .unwrap();

    if let Some(local_path) = config::find_local_config() {
        writeln!(
            out,
            "  {command}{:10}{command:#} \t{}",
            "Local:",
            local_path.display()
        )
        .unwrap();
    }

    writeln!(out, "\n{header}Commands:{header:#}").unwrap();

    let mut keys: Vec<&String> = cfg.commands.keys().collect();
    keys.sort();

    writeln!(out, "{command}  app{command:#}  \t\t[builtin]").unwrap();
    if keys.is_empty() {
        writeln!(out, "  {command}(no commands configured){command:#}").unwrap();
    } else {
        for key in keys {
            if let Some(cmd) = cfg.commands.get(key) {
                writeln!(
                    out,
                    "  {command}{key}{command:#} \t{:>}",
                    format_cmd_type(cmd),
                )
                .unwrap();
            }
        }
    }
    out
}

pub fn format_cmd_type(cmd: &Command) -> String {
    match cmd.kind {
        ExecutionMode::Shell => "[shell]".to_string(),
        ExecutionMode::TemplateShell => "[template_shell]".to_string(),
    }
}
