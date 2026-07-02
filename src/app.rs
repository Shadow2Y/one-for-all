use anyhow::Result;
use clap::{Parser, Subcommand};

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

pub fn handle(args: &[String]) -> Result<String> {
    Ok(args[0].to_string())
}

pub fn help() -> Result<String> {
    println!("HELP");
    Ok(String::new())
}
