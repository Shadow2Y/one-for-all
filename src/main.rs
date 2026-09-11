use std::io::Error;

use clap::Parser;

mod app;
mod config;
mod engine;
mod models;

#[derive(Parser)]
#[command(
    name = "ofa",
    about = "The one CLI tool to orchestrate them all",
    version,
    arg_required_else_help = false
)]
struct CLI {
    /// The command to run (built-in or from project config).
    command: Option<String>,

    /// Additional arguments forwarded to the command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let cli = CLI::parse();

    let output = match cli.command.as_deref() {
        Some("app") => app::handle(&cli.args),
        Some(cmd) => engine::handle_command(cmd, &cli.args),
        None => app::help(),
    }?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        return Err(Error::other(format!("command failed: {}", output.status)));
    }

    Ok(())
}
