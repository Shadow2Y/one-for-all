use clap::Parser;
use std::io::{Error, Result};

use crate::models::request::ExecutionRequest;

mod app;
mod config;
mod engine;
mod models;

#[derive(Parser, Debug)]
#[command(
    version,
    name = "ofa",
    after_help = app::help(),
    arg_required_else_help = true,
    about = "\n\n`one-for-all` the one CLI tool for orchestrating them all"
)]
struct CLI {
    /// Dry run mode -- print the resultant script without executing it
    #[arg(long, global = true)]
    dry_run: bool,

    /// Verbose mode -- write extra information regarding the current execution
    #[arg(long, global = true)]
    verbose: bool,

    /// Variable interpolation --
    #[arg(long, global = true)]
    interpolate: bool,

    /// Everything else (command + arguments)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = CLI::parse();

    let (command, args) = match cli.args.split_first() {
        Some((cmd, rest)) => (Some(cmd.as_str()), rest.to_vec()),
        None => (None, Vec::new()),
    };

    let output = match command {
        Some("app") => app::handle(&args),

        Some(cmd) => {
            let command = config::get()
                .commands
                .get(cmd)
                .expect("Unknown command")
                .clone();

            engine::run(ExecutionRequest {
                dry_run: cli.dry_run,
                verbose: cli.verbose,
                interpolate: cli.interpolate,
                cmd: command,
                args,
            })
        }
        None => panic!("Unsupported operation!"),
    }?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(Error::other(format!("command failed: {}", output.status)));
    }

    Ok(())
}
