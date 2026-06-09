use std::io::Error;

use clap::Parser;

mod config;
mod init;
mod lifecycle;

#[derive(Parser)]
#[command(name = "ofa")]
struct CLI {
    command: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    let cli = CLI::parse();

    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let result = match cli.command.as_deref() {
        Some("init") => init::handle_init(),
        Some("help") | None => display_help(),
        Some(lifecycle) => {
            config::init();
            lifecycle::handle_lifecycle(lifecycle, cli.args)
        }
    };

    match result {
        Ok(()) => {}
        Err(e) => {
            log::error!("Process failed with the following error :: {}", e);
            std::process::exit(1)
        }
    }
}

fn display_help() -> Result<(), Error> {
    Ok(())
}
