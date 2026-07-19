use anyhow::{Result, bail};

use crate::{
    config,
    context::{self, registry::CommandRegistry},
    engine::tokenizer::{Template, TemplatePart},
    models::{
        Value,
        command::{Command, CommandKind, ExecutionMode},
        value::output_to_value,
        variable::{Provider, Variable},
    },
};
use std::process::Command as ProcessCommand;

pub fn execute_command(
    registry: &'static CommandRegistry,
    cmd: &Command,
    args: &[String],
) -> Result<Value> {
    match &cmd.cmd {
        CommandKind::Script(script) => execute_script(registry, &cmd.kind, script, args),

        CommandKind::Args(cmd_args) => {
            execute_script(registry, &cmd.kind, &cmd_args.join(" "), args)
        }

        CommandKind::Parameterized(func) => {
            if args.len() < func.params.len() {
                bail!(
                    "Expected {} arguments but got {}",
                    func.params.len(),
                    args.len()
                );
            }

            for (name, value) in func.params.iter().zip(args.iter()) {
                context::set(name.clone(), Value::String(value.clone()));
            }

            execute_script(registry, &cmd.kind, &func.run, args)
        }

        CommandKind::Group(group) => {
            bail!(
                "Unsupported execution request of type: Group :: {:?}",
                group
            )
        }
    }
}

pub fn execute_script(
    registry: &'static CommandRegistry,
    executor: &ExecutionMode,
    script: &str,
    args: &[String],
) -> Result<Value> {
    executor.execute(registry, script, args)
}

fn resolve_template(registry: &'static CommandRegistry, text: &str) -> Result<String> {
    let template = Template::parse(text)?;
    let mut result = String::new();

    for part in &template.parts {
        match part {
            TemplatePart::Text(t) => result.push_str(t),
            TemplatePart::Expr(e) => {
                let value = e.resolve(registry)?;
                result.push_str(&value.to_string());
            }
        }
    }

    Ok(result)
}

impl ExecutionMode {
    pub fn execute(
        &self,
        registry: &'static CommandRegistry,
        script: &str,
        args: &[String],
    ) -> Result<Value> {
        match self {
            Self::Shell => execute_shell(script, args),

            Self::Template => Ok(Value::String(resolve_template(registry, script)?)),

            Self::TemplateShell => {
                let script = resolve_template(registry, script)?;
                execute_shell(&script, args)
            }
        }
    }
}

fn execute_shell(script: &str, args: &[String]) -> Result<Value> {
    let env_vars = config::get().env.clone();

    let output = ProcessCommand::new("sh")
        .arg("-c")
        .arg(script)
        .arg("--")
        .args(args)
        .envs(env_vars)
        .output()?;

    Ok(output_to_value(output))
}
