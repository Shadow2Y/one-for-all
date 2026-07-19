use std::collections::HashMap;
use std::process::Command as ProcessCommand;

use anyhow::{Result, bail};

use crate::{
    config,
    context::registry::FunctionRegistry,
    engine::resolver,
    models::{
        Value,
        command::{Command, CommandKind, ExecutionMode},
        value::output_to_value,
    },
};

// ── Public entry point ────────────────────────────────────────────────────────

/// Dispatches a resolved [`Command`] to the appropriate execution path.
///
/// `local_vars` carries call-scoped variables (e.g. parameterised command
/// arguments) so that they never touch shared global state, keeping concurrent
/// executions isolated.
pub fn execute_command(
    registry: &'static FunctionRegistry,
    cmd: &Command,
    args: &[String],
) -> Result<Value> {
    execute_with_scope(registry, cmd, args, &HashMap::new())
}

// ── Internal dispatcher ───────────────────────────────────────────────────────

fn execute_with_scope(
    registry: &'static FunctionRegistry,
    cmd: &Command,
    args: &[String],
    local_vars: &HashMap<String, Value>,
) -> Result<Value> {
    match &cmd.cmd {
        CommandKind::Script(script) => run(registry, &cmd.kind, script, args, local_vars),

        CommandKind::Args(parts) => run(registry, &cmd.kind, &parts.join(" "), args, local_vars),

        CommandKind::Parameterized(func) => {
            if args.len() < func.params.len() {
                bail!(
                    "Expected {} argument(s) but got {}",
                    func.params.len(),
                    args.len()
                );
            }

            // Build a call-local var scope from the named parameters.
            // These are passed down into the resolver instead of being written
            // to the global / thread-local store, so concurrent calls to the
            // same parameterised command cannot interfere with each other.
            let params: HashMap<String, Value> = func
                .params
                .iter()
                .zip(args.iter())
                .map(|(name, val)| (name.clone(), Value::String(val.clone())))
                .collect();

            run(registry, &cmd.kind, &func.run, args, &params)
        }

        CommandKind::Group(children) => {
            bail!(
                "Cannot execute a command group directly — subcommand required. \
                 Available: {:?}",
                children.keys().collect::<Vec<_>>()
            )
        }
    }
}

// ── Execution modes ───────────────────────────────────────────────────────────

/// Runs a script string under the given [`ExecutionMode`], applying template
/// resolution where needed.
fn run(
    registry: &'static FunctionRegistry,
    mode: &ExecutionMode,
    script: &str,
    args: &[String],
    local_vars: &HashMap<String, Value>,
) -> Result<Value> {
    match mode {
        ExecutionMode::Shell => shell(script, args),

        ExecutionMode::Template => {
            let rendered = resolver::render(script, registry, local_vars)?;
            Ok(Value::String(rendered))
        }

        ExecutionMode::TemplateShell => {
            let rendered = resolver::render(script, registry, local_vars)?;
            shell(&rendered, args)
        }
    }
}

// ── Shell execution ───────────────────────────────────────────────────────────

fn shell(script: &str, args: &[String]) -> Result<Value> {
    let output = ProcessCommand::new("sh")
        .arg("-c")
        .arg(script)
        .arg("--")
        .args(args)
        .envs(config::get().env.clone())
        .output()?;

    Ok(output_to_value(output))
}
