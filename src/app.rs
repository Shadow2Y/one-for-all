use std::any::Any;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::models::Value;

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
    Ok(Value::String(args[0].to_string()))
}

pub fn help() -> Result<Value> {
    Ok(Value::String("HELP".to_string()))
}
