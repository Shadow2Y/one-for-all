use std::{path::PathBuf, sync::LazyLock};
mod persistent;
mod runtime;
use crate::context::registry::CommandRegistry;
pub mod defaults;
pub mod registry;
pub mod store;
pub use defaults::get;
pub use defaults::set;

static REGISTRY: LazyLock<CommandRegistry> = LazyLock::new(|| init());

pub fn get_registry() -> &'static CommandRegistry {
    &REGISTRY
}

pub fn init() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    registry.register_func("env", defaults::dyn_env);
    registry.register_func("add", defaults::dyn_add);
    registry.register_func("sub", defaults::dyn_sub);
    registry.register_func("mul", defaults::dyn_mul);
    registry.register_func("div", defaults::dyn_div);
    registry.register_func("throw", defaults::throw);
    registry.register_func("print", defaults::println);
    registry.register_func("eq", defaults::dyn_equals);
    registry.register_func("mod", defaults::dyn_modulus);
    registry.register_func("sup", defaults::suppress);

    registry.register_func("set", defaults::dyn_set);
    registry.register_func("get", defaults::dyn_get);
    registry.register_func("store", defaults::dyn_store);
    registry
}

pub fn current() -> String {
    let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));

    cwd.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
