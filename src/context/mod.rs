use std::sync::LazyLock;

use anyhow::bail;
pub use context::ExecutionContext;

use crate::context::store::Store;

pub mod context;
mod store;

static CONTEXT: LazyLock<ExecutionContext> = LazyLock::new(|| {
    let store = Store::new("~/.ofa/global.toml");
    let mut context = ExecutionContext::new();

    context.register_func("store", move |_ctx, _args| store.get());
    context.register_func("set_val", |ctx, _args| {
        ctx.set_var("name", "value");
        Ok(String::new())
    });

    context.register_func("add", |_ctx, args| {
        if args.len() != 2 {
            bail!("add requires 2 arguments".to_string());
        }
        let a: i32 = args[0].parse().map_err(|_| "Invalid number").expect("msg");
        let b: i32 = args[1].parse().map_err(|_| "Invalid number").expect("msg");
        Ok((a + b).to_string())
    });

    context
});

pub fn get() -> &'static ExecutionContext {
    &CONTEXT
}
