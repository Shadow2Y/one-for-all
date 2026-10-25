use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output, Stdio};

use crate::models::request::ExecutionRequest;
use crate::{config, engine::resolver, models::command::ExecutionMode};

static NOHUP_DIR: &str = "nohup.out";

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
    let envs = config::get().env.clone();
    let script = if request.interpolate {
        interpolate(script, &envs)
    } else {
        script.to_owned()
    };
    if request.dry_run {
        return execute_dry(&script, request);
    }
    if request.verbose {
        println!("Executing :: {script}");
    }
    if request.expose_env {
        println!("{}\n", formatted_env());
    }

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(script);

    if !request.args.is_empty() {
        cmd.arg("--").args(&request.args);
    }
    cmd.envs(envs);

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
            .open(NOHUP_DIR)?;
        let err_file = out_file.try_clone()?;

        (Stdio::from(out_file), Stdio::from(err_file))
    };

    cmd.stdin(Stdio::null()).stdout(stdout).stderr(stderr);

    #[cfg(unix)]
    {
        use std::env;

        use crate::config::pid_dir;

        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let process_id = cmd.spawn()?.id();
        let pid_path = pid_dir();
        persist_pid(&process_id, cmd.get_args(), &env::current_dir()?, &pid_path)?;
        println!(
            "PID :: {process_id} :: PID-history :: {:?} :: output-dir :: {NOHUP_DIR}\n",
            pid_path
        );

        Ok(Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: process_id.to_string().into_bytes(),
            stderr: Vec::new(),
        })
    }

    #[cfg(not(unix))]
    Err(anyhow::anyhow!(
        "nohup mode is only supported on Unix systems"
    ))
}

fn persist_pid<'a>(
    pid: &u32,
    args: impl Iterator<Item = &'a OsStr>,
    current_dir: &PathBuf,
    persist_dir: &PathBuf,
) -> std::io::Result<()> {
    let args_str: Vec<String> = args.map(|arg| arg.to_string_lossy().into_owned()).collect();

    let command_line = args_str.join(" ");

    let mut pid_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(persist_dir)?;

    writeln!(pid_file, "{pid}\t{current_dir:?}\t{command_line}")?;
    Ok(())
}

fn execute_dry(script: &str, request: &ExecutionRequest) -> Result<Output> {
    let output: String;
    if request.expose_env {
        let env_str = formatted_env();
        output = format!("{env_str} {script}\n");
    } else {
        output = format!("{script}\n");
    }
    Ok(Output {
        status: ExitStatus::default(),
        stdout: output.into_bytes(),
        stderr: Vec::new(),
    })
}

fn formatted_env() -> String {
    config::get()
        .env
        .iter()
        .map(|(k, v)| format!("{}='{}'", k, v))
        .collect::<Vec<String>>()
        .join(" ")
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
