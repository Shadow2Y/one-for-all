use std::{collections::HashMap, io::Error};

use crate::{
    config,
    engine::tokenizer::{Template, TemplatePart},
    models::{request::ExecutionRequest, variable::Variable},
};

// ── Template resolution ───────────────────────────────────────────────────────

/// Renders a [`Template`] into a `String` by resolving every [`Expr`] it
/// contains.
///
/// `local_vars` is checked first (e.g. the params of a parameterised command),
/// then the thread-local session store, then the on-disk persistent store,
/// and finally the config `[vars]` table.  This layered priority means that
/// callers never need to pollute global state — they just pass their scope
/// inline and resolution stays deterministic across concurrent executions.
pub fn render_template(
    template: &Template,
    local_vars: &HashMap<&str, &str>,
) -> Result<String, Error> {
    let mut out = String::new();
    for part in &template.parts {
        match part {
            TemplatePart::Text(t) => out.push_str(t),
            TemplatePart::Expr(e) => out.push_str(&resolve_var(e, local_vars)?.to_string()),
        }
    }
    Ok(out)
}

/// Convenience wrapper: parse `text` into a [`Template`] then render it.
pub fn render_text(text: &str, local_vars: &HashMap<&str, &str>) -> Result<String, Error> {
    render_template(&Template::parse(text)?, local_vars)
}

// ── Variable resolution ───────────────────────────────────────────────────────

/// Resolves a named variable following the layered lookup order described in
/// [`resolve_expr`].
fn resolve_var(name: &str, local_vars: &HashMap<&str, &str>) -> Result<String, Error> {
    // 1. Call-local scope (e.g. parameterised command arguments)
    if let Some(v) = local_vars.get(name) {
        return Ok(v.to_string());
    }

    // 3. Config [vars] table — may be a literal value or a provider command
    if let Some(variable) = config::get().vars.get(name) {
        return resolve_variable(variable);
    }

    Err(Error::new(
        std::io::ErrorKind::NotFound,
        format!("Undefined variable '{name}'"),
    ))
}

/// Resolves a [`Variable`] from the config `[vars]` table.
///
/// - `Variable::Literal` → returned directly, zero execution cost.
/// - `Variable::Provided` → the inner `Command` is executed to produce the
///   value. Provider commands run with an empty `local_vars` scope because they
///   are config-level, not call-scoped.
pub fn resolve_variable(variable: &Variable) -> Result<String, Error> {
    match variable {
        Variable::Literal(v) => Ok(v.clone()),
        Variable::Provided(provider) => {
            // Providers run in an isolated scope — no local vars bleed in.
            let output = crate::engine::execute_command(ExecutionRequest::new(
                provider.run.to_owned(),
                Vec::new(),
            ));
            Ok(String::from_utf8_lossy(&output?.stdout).into_owned())
        }
    }
}
