use anyhow::Result;
use clap::Parser;

mod config;
mod engine;
mod init;
mod store;

// ── CLI definition ────────────────────────────────────────────────────────────
//
// ofa uses a hybrid dispatch model:
//   - "init" is a built-in command handled before touching the config.
//   - Everything else is looked up in the resolved project config.
//
// Trailing args are forwarded to the resolved runnable so you can do things
// like `ofa test -- --nocapture`.

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

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let cli = CLI::parse();

    let result: Result<()> = match cli.command.as_deref() {
        // Built-in: init [--global]
        Some("init") => {
            let global = cli.args.iter().any(|a| a == "--global");
            init::handle_init(global)
        }

        // Anything else → look up in config and execute.
        Some(cmd) => engine::handle_command(cmd, cli.args),

        // No command → show help with available project commands.
        None => display_help(),
    };

    if let Err(e) = result {
        log::error!("{:#}", e);
        std::process::exit(1);
    }
}

// ── Help ──────────────────────────────────────────────────────────────────────

fn display_help() -> Result<()> {
    println!("ofa — the one CLI tool to orchestrate them all\n");
    println!("USAGE:");
    println!("  ofa <command> [args...]\n");
    println!("BUILT-IN COMMANDS:");
    println!("  init           Initialize ofa in the current project");
    println!("  init --global  Install the global config at ~/.ofa/config.json");

    // If a project config exists, list its commands.
    if let Some(cfg) = config::try_get_config() {
        if !cfg.commands.is_empty() {
            println!("\nPROJECT COMMANDS:");
            let mut cmds: Vec<&String> = cfg.commands.keys().collect();
            cmds.sort();
            for cmd in cmds {
                println!("  {}", cmd);
            }
        }
        if !cfg.runnables.is_empty() {
            println!("\nRUNNABLES:");
            let mut runnables: Vec<&String> = cfg.runnables.keys().collect();
            runnables.sort();
            for r in runnables {
                println!("  #{}", r);
            }
        }
    } else {
        println!("\n  Run `ofa init` to set up this project.");
    }

    Ok(())
}
