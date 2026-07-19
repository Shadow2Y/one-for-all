use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::{
    config,
    context::{registry::FunctionRegistry, store},
    engine::tokenizer::{Expr, Template, TemplatePart},
    models::{Value, variable::Variable},
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
    registry: &'static FunctionRegistry,
    local_vars: &HashMap<String, Value>,
) -> Result<String> {
    let mut out = String::new();
    for part in &template.parts {
        match part {
            TemplatePart::Text(t) => out.push_str(t),
            TemplatePart::Expr(e) => {
                out.push_str(&resolve_expr(e, registry, local_vars)?.to_string())
            }
        }
    }
    Ok(out)
}

/// Convenience wrapper: parse `text` into a [`Template`] then render it.
pub fn render(
    text: &str,
    registry: &'static FunctionRegistry,
    local_vars: &HashMap<String, Value>,
) -> Result<String> {
    render_template(&Template::parse(text)?, registry, local_vars)
}

// ── Expression resolution ─────────────────────────────────────────────────────

/// Resolves an [`Expr`] to a [`Value`].
///
/// Variable lookup order (first hit wins):
/// 1. `local_vars`          — call-scoped (e.g. parameterised command params)
/// 2. Thread-local session  — `set()`/`get()` builtins within the current thread
/// 3. Persistent store      — `store()` values (TOML-backed, per-namespace)
/// 4. Config `[vars]` table — static or provider-backed variables
pub fn resolve_expr(
    expr: &Expr,
    registry: &'static FunctionRegistry,
    local_vars: &HashMap<String, Value>,
) -> Result<Value> {
    match expr {
        Expr::Literal(lit) => Ok(lit.into()),

        Expr::Var(name) => resolve_var(name, registry, local_vars),

        Expr::Func(name, arg_exprs) => {
            let args: Result<Vec<Value>> = arg_exprs
                .iter()
                .map(|e| resolve_expr(e, registry, local_vars))
                .collect();
            registry.execute_func(name, &args?)
        }
    }
}

// ── Variable resolution ───────────────────────────────────────────────────────

/// Resolves a named variable following the layered lookup order described in
/// [`resolve_expr`].
fn resolve_var(
    name: &str,
    registry: &'static FunctionRegistry,
    local_vars: &HashMap<String, Value>,
) -> Result<Value> {
    // 1. Call-local scope (e.g. parameterised command arguments)
    if let Some(v) = local_vars.get(name) {
        return Ok(v.clone());
    }

    // 2. Thread-local session + persistent store
    if let Some(v) = store::fetch(name) {
        return Ok(v);
    }

    // 3. Config [vars] table — may be a literal value or a provider command
    if let Some(variable) = config::get().vars.get(name) {
        return resolve_variable(variable, registry);
    }

    bail!("Undefined variable '{name}'")
}

/// Resolves a [`Variable`] from the config `[vars]` table.
///
/// - `Variable::Literal` → returned directly, zero execution cost.
/// - `Variable::Provided` → the inner `Command` is executed to produce the
///   value. Provider commands run with an empty `local_vars` scope because they
///   are config-level, not call-scoped.
pub fn resolve_variable(variable: &Variable, registry: &'static FunctionRegistry) -> Result<Value> {
    match variable {
        Variable::Literal(v) => Ok(v.clone()),
        Variable::Provided(provider) => {
            // Providers run in an isolated scope — no local vars bleed in.
            crate::engine::execute_command(registry, &provider.run, &[])
        }
    }
}
