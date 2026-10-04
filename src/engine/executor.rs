use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Result};
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::models::request::ExecutionRequest;
use crate::{config, engine::resolver, models::command::ExecutionMode};

// ── Internal dispatcher ───────────────────────────────────────────────────────

pub fn execute_command(mut request: ExecutionRequest) -> Result<Output> {
    let func = &request.cmd;
    if request.args.len() < func.params.len() {
        let missing_params: Vec<&str> = func.params[request.args.len()..]
            .iter()
            .filter(|name| !func.defaults.contains_key(*name))
            .map(String::as_str)
            .collect();

        if !missing_params.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Missing required parameter :: [ {} ]",
                    missing_params.join(", ")
                ),
            ));
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

    run(&func.cmd, &request, &params)
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

#[cfg(unix)]
use std::os::unix::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub fn execute(script: &str, request: &ExecutionRequest) -> Result<Output> {
    if request.dry_run {
        return execute_dry(script, request);
    }
    if request.verbose {
        println!("Executing :: {script}");
    }

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(script);

    if !request.args.is_empty() {
        cmd.arg("--").args(&request.args);
    }
    cmd.envs(config::get().env.clone());

    if request.nohup {
        spawn_detached(cmd, request.quiet)
    } else {
        spawn_attached(cmd, request.quiet)
    }
}

fn spawn_attached(mut cmd: Command, quiet: bool) -> Result<Output> {
    let (stdout, stderr) = if quiet {
        (Stdio::null(), Stdio::null())
    } else {
        (Stdio::inherit(), Stdio::inherit())
    };

    cmd.stdout(stdout)
        .stderr(stderr)
        .output()
        .map_err(Into::into)
}

fn spawn_detached(mut cmd: Command, quiet: bool) -> Result<Output> {
    let (stdout, stderr) = if quiet {
        (Stdio::null(), Stdio::null())
    } else {
        let out_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("nohup.out")?;
        let err_file = out_file.try_clone()?;

        (Stdio::from(out_file), Stdio::from(err_file))
    };

    cmd.stdin(Stdio::null()).stdout(stdout).stderr(stderr);

    #[cfg(unix)]
    {
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let process = cmd.spawn()?.id();
        println!("PID :: {process}");

        Ok(Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
    }

    #[cfg(not(unix))]
    Err(anyhow::anyhow!(
        "nohup mode is only supported on Unix systems"
    ))
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
