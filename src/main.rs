use crate::context::context::ExecutionContext;
use crate::engine::tokenizer::Expr;
use anyhow::{Result, bail};
use clap::Parser;

mod app;
mod config;
mod context;
mod engine;
mod models;

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
    // mainyo();

    // env_logger::Builder::from_default_env()
    //     .format_timestamp(None)
    //     .init();

    let cli = CLI::parse();

    let result: Result<String> = match cli.command.as_deref() {
        Some("app") => app::handle(&cli.args),
        Some(cmd) => engine::handle_command(cmd, &cli.args),
        None => app::help(),
    };

    match result {
        Ok(val) => println!("{val}"),
        Err(e) => {
            log::error!("{:#}", e);
            std::process::exit(1);
        }
    }
}

fn mainyo() {
    // 1. Initialize the registry
    let mut context = ExecutionContext::new();

    // 2. Set your variables
    context.set_var("a", "20");
    context.set_var("b", "5");
    context.set_var("c", "3");

    // 4. Parse the custom string
    let expression_string = "!add(!sub(#a,#b), #c)";
    println!("Parsing: {}", expression_string);

    let ast = Expr::parse(expression_string).expect("Failed to parse expression");

    // 5. Execute it!
    match ast.resolve(&context) {
        Ok(result) => println!("Result: {}", result), // (20 - 5) + 3 = 18
        Err(e) => println!("Execution Error: {}", e),
    }
}
