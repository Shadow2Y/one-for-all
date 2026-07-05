use anyhow::Result;

use crate::{
    context::registry::CommandRegistry,
    engine::tokenizer::{Template, TemplatePart},
    models::Value,
};
pub fn execute_command(registry: &'static CommandRegistry, cmd: &String) -> Result<Value> {
    log::debug!("{:?}", std::env::args().collect::<Vec<_>>());

    if cmd.is_empty() {
        return Ok(Value::Void);
    }

    let template = Template::parse(cmd)?;
    let mut result = String::new();

    for part in &template.parts {
        match part {
            TemplatePart::Text(t) => result.push_str(t),
            TemplatePart::Expr(e) => {
                let value = e.resolve(&registry)?;
                result.push_str(&value.to_string());
            }
        }
    }

    Ok(Value::String(result))
}
