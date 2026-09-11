use std::{
    io::Error,
    process::{ExitStatus, Output},
};

use clap::{Parser, Subcommand};

use crate::{
    engine::{self, discovery},
    models::command::{Command, CommandKind, ExecutionMode::TemplateShell},
};

#[derive(Parser)]
#[command(
    name = "ofa app",
    about = "Manage ofa internals",
    arg_required_else_help = true
)]
struct App {
    #[command(subcommand)]
    cmd: AppCmd,
}

#[derive(Subcommand)]
enum AppCmd {
    Help,
}

pub fn handle(args: &[String]) -> Result<Output, Error> {
    let (cmd, args) = args
        .split_first()
        .ok_or_else(|| Error::new(std::io::ErrorKind::InvalidInput, "missing command"))?;
    if cmd == "eval" {
        engine::execute_command(
            &Command {
                kind: TemplateShell,
                cmd: CommandKind::Args(args.to_vec()),
            },
            &[],
        )
    } else {
        help()
    }
}

pub fn help() -> Result<Output, Error> {
    Ok(Output {
        status: ExitStatus::default(),
        stdout: discovery::list_base_commands().into_bytes(),
        stderr: Vec::new(),
    })
}
