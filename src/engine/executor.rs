use std::io::{Error, ErrorKind, Result};
use std::process::{Command as ProcessCommand, ExitStatus};
use std::{collections::HashMap, process::Output};

use crate::models::request::ExecutionRequest;
use crate::{
    config,
    engine::resolver,
    models::command::{CommandKind, ExecutionMode},
};

// ── Internal dispatcher ───────────────────────────────────────────────────────

pub fn execute_command(mut request: ExecutionRequest) -> Result<Output> {
    match &request.cmd.cmd {
        CommandKind::Script(script) => run(script, &request, &HashMap::new()),

        CommandKind::Args(parts) => run(&parts.join(" "), &request, &HashMap::new()),

        CommandKind::Parameterized(func) => {
            if request.args.len() < func.params.len() {
                for name in &func.params[request.args.len()..] {
                    if !func.defaults.contains_key(name) {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            format!("Missing required parameter: {name}"),
                        ));
                    }
                }
            }

            if !func.allow_trailing_args {
                request.args.truncate(func.params.len());
            }

            let params: HashMap<&str, &str> = func
                .params
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let value = request
                        .args
                        .get(index)
                        .map(String::as_str)
                        .or_else(|| func.defaults.get(name).map(String::as_str))
                        .expect("parameter was validated above");

                    (name.as_str(), value)
                })
                .collect();

            run(&func.run, &request, &params)
        }

        CommandKind::Group(children) => Err(Error::new(
            ErrorKind::Unsupported,
            format!(
                "Cannot execute a command group directly — subcommand required. Available: {:?}",
                children.keys().collect::<Vec<_>>()
            ),
        )),
    }
}

// ── Execution modes ───────────────────────────────────────────────────────────

/// Runs a script string under the given [`ExecutionMode`], applying template
/// resolution where needed.
fn run(
    script: &str,
    request: &ExecutionRequest,
    local_vars: &HashMap<&str, &str>,
) -> Result<Output> {
    match request.cmd.kind {
        ExecutionMode::Shell => execute(script, request),

        ExecutionMode::TemplateShell => {
            let rendered = resolver::render_text(&script, local_vars)?;
            execute(&rendered, request)
        }
    }
}

fn execute(script: &str, request: &ExecutionRequest) -> Result<Output> {
    if request.dry_run {
        return execute_dry(script, request);
    }
    if request.verbose {
        println!("Executing :: {script}")
    }
    ProcessCommand::new("sh")
        .arg("-c")
        .arg(script)
        .arg("--")
        .args(request.args.to_owned())
        .envs(config::get().env.clone())
        .output()
}

fn execute_dry(script: &str, request: &ExecutionRequest) -> Result<Output> {
    let script = if request.interpolate {
        interpolate(script, &config::get().env)
    } else {
        script.to_owned()
    };

    Ok(Output {
        status: ExitStatus::default(),
        stdout: format!("{script}\n").into_bytes(),
        stderr: Vec::new(),
    })
}

fn interpolate(input: &str, env: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
            continue;
        }

        let mut name = String::new();

        while let Some(&c) = chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                name.push(c);
                chars.next();
            } else {
                break;
            }
        }

        if name.is_empty() {
            result.push('$');
        } else if let Some(value) = env.get(&name) {
            result.push_str(value);
        } else {
            // Keep unknown variables unchanged
            result.push('$');
            result.push_str(&name);
        }
    }

    result
}
