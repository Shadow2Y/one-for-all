use std::io::Error;
use std::process::Command as ProcessCommand;
use std::{collections::HashMap, process::Output};

use crate::{
    config,
    engine::resolver,
    models::command::{Command, CommandKind, ExecutionMode},
};

// ── Internal dispatcher ───────────────────────────────────────────────────────

pub fn execute_command(cmd: &Command, args: &[String]) -> Result<Output, Error> {
    match &cmd.cmd {
        CommandKind::Script(script) => run(&cmd.kind, script, args, &HashMap::new()),

        CommandKind::Args(parts) => run(&cmd.kind, &parts.join(" "), args, &HashMap::new()),

        CommandKind::Parameterized(func) => {
            if args.len() < func.params.len() {
                panic!(
                    "Expected {} argument(s) but got {}",
                    func.params.len(),
                    args.len()
                );
            }

            let params: HashMap<String, String> = func
                .params
                .iter()
                .zip(args.iter())
                .map(|(name, val)| (name.clone(), val.clone()))
                .collect();

            run(&cmd.kind, &func.run, args, &params)
        }

        CommandKind::Group(children) => {
            panic!(
                "Cannot execute a command group directly — subcommand required. Available: {:?}",
                children.keys().collect::<Vec<_>>()
            )
        }
    }
}
// ── Execution modes ───────────────────────────────────────────────────────────

/// Runs a script string under the given [`ExecutionMode`], applying template
/// resolution where needed.
fn run(
    mode: &ExecutionMode,
    script: &str,
    args: &[String],
    local_vars: &HashMap<String, String>,
) -> Result<Output, Error> {
    match mode {
        ExecutionMode::Shell => shell(script, args),

        ExecutionMode::TemplateShell => {
            let rendered = resolver::render_text(script, local_vars)?;
            shell(&rendered, args)
        }
    }
}

// ── Shell execution ───────────────────────────────────────────────────────────

fn shell(script: &str, args: &[String]) -> Result<Output, Error> {
    ProcessCommand::new("sh")
        .arg("-c")
        .arg(script)
        .arg("--")
        .args(args)
        .envs(config::get().env.clone())
        .output()
}
