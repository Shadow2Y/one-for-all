use std::{path::PathBuf, sync::LazyLock};

mod builtins;
mod persistent;
pub mod registry;
mod runtime;
pub mod store;

use crate::context::registry::FunctionRegistry;

// ── Global function registry ──────────────────────────────────────────────────

static REGISTRY: LazyLock<FunctionRegistry> = LazyLock::new(init);

pub fn get_registry() -> &'static FunctionRegistry {
    &REGISTRY
}

fn init() -> FunctionRegistry {
    let mut r = FunctionRegistry::new();

    // Arithmetic
    r.register_func("add", builtins::dyn_add);
    r.register_func("sub", builtins::dyn_sub);
    r.register_func("mul", builtins::dyn_mul);
    r.register_func("div", builtins::dyn_div);
    r.register_func("mod", builtins::dyn_modulus);

    // Comparison
    r.register_func("eq", builtins::dyn_equals);

    // Environment
    r.register_func("env", builtins::dyn_env);

    // Control / side-effects
    r.register_func("print", builtins::println);
    r.register_func("throw", builtins::throw);
    r.register_func("sup", builtins::suppress);

    // Variable store
    r.register_func("set", builtins::dyn_set);
    r.register_func("get", builtins::dyn_get);
    r.register_func("store", builtins::dyn_store);

    r
}

// ── Utilities ─────────────────────────────────────────────────────────────────

/// Returns the name of the current working directory (used as the store
/// namespace so that persistent variables are scoped per-project).
pub fn current() -> String {
    let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));
    cwd.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
