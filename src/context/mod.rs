pub use context::ExecutionContext;
use std::sync::LazyLock;

use crate::context::registry::CommandRegistry;

pub mod context;
pub mod defaults;
pub mod registry;
mod store;

static CONTEXT: LazyLock<ExecutionContext> = LazyLock::new(|| ExecutionContext::new());
static REGISTRY: LazyLock<CommandRegistry> = LazyLock::new(|| init());

pub fn get_context() -> &'static ExecutionContext {
    &CONTEXT
}

pub fn get_registry() -> &'static CommandRegistry {
    &REGISTRY
}

pub fn init() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register_func("add", defaults::dyn_add);
    registry.register_func("mul", defaults::dyn_mul);
    registry
}
