use anyhow::{Result, bail};

use crate::{
    config,
    context::{self, registry::CommandRegistry},
    engine::tokenizer::{Template, TemplatePart},
    models::{Value, command::Command, value::output_to_value},
};
use std::process::Command as ProcessCommand;

pub fn execute_command(
    registry: &'static CommandRegistry,
    cmd: &Command,
    args: &[String],
) -> Result<Value> {
    match cmd {
        Command::Script(script) => execute_script(registry, script, false),

        Command::Args(cmd_args) => execute_script(registry, cmd_args.join(" ").as_str(), true),

        Command::Parameterized(func) => {
            if args.len() < func.params.len() {
                bail!(
                    "Expected {} arguments but got {}",
                    func.params.len(),
                    args.len()
                );
            }

            for (name, value) in func.params.iter().zip(args.iter()) {
                context::store::set(name.clone(), Value::String(value.clone()));
            }

            execute_script(registry, &func.run, true)
        }

        Command::Group(group) => {
            bail!(
                "Unsupported execution request of type: Group :: {:?}",
                group
            )
        }
    }
}

pub fn execute_script(
    registry: &'static CommandRegistry,
    script: &str,
    exec_shell: bool,
) -> Result<Value> {
    let script = resolve_template(registry, script)?;
    let env_vars = config::get().env.clone();
    if !exec_shell {
        return Ok(Value::String(script));
    }
    let output = ProcessCommand::new("sh")
        .arg("-c")
        .envs(env_vars)
        .arg(&script)
        .output()?;

    Ok(output_to_value(output))
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
