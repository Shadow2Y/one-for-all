use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use crate::{
    context,
    engine::{self, discovery},
    models::{
        Value,
        command::{Command, CommandKind, ExecutionMode::TemplateShell},
    },
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

pub fn handle(args: &[String]) -> Result<Value> {
    let (cmd, _args) = args.split_first().expect("msg");

    if cmd == "eval" {
        // If the input string was "eval", execute it with the args
        engine::execute_command(
            context::get_registry(),
            &Command {
                kind: TemplateShell,
                cmd: CommandKind::Args(_args.to_vec()),
            },
            &[],
        )
    } else {
        bail!("Err, unsupported cmd")
    }
}

pub fn help() -> Result<Value> {
    Ok(discovery::list_base_commands())
}
