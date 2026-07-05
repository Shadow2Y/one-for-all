use std::sync::LazyLock;

pub use self::store::get;
use crate::context::registry::CommandRegistry;
pub mod defaults;
pub mod registry;
mod store;

static REGISTRY: LazyLock<CommandRegistry> = LazyLock::new(|| init());

pub fn get_registry() -> &'static CommandRegistry {
    &REGISTRY
}

pub fn init() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register_func("add", defaults::dyn_add);
    registry.register_func("sub", defaults::dyn_sub);
    registry.register_func("mul", defaults::dyn_mul);
    registry.register_func("div", defaults::dyn_div);
    registry.register_func("throw", defaults::throw);
    registry.register_func("print", defaults::println);
    registry.register_func("eq", defaults::dyn_equals);
    registry.register_func("mod", defaults::dyn_modulus);
    registry.register_func("sup", defaults::suppress);
    registry.register_func("set", store::dyn_set);
    registry.register_func("get", store::dyn_get);
    registry
}
