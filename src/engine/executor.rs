use anyhow::Result;

use crate::{
    context::{ExecutionContext, registry::CommandRegistry},
    engine::tokenizer::Expr,
    models::Value,
};

pub fn execute_command(
    context: &'static ExecutionContext,
    registry: &'static CommandRegistry,
    args: &[String],
) -> Result<Value> {
    log::debug!("{:?}", std::env::args().collect::<Vec<_>>());
    let ast = Expr::parse(&args[0]).expect("Failed to parse expression");

    ast.resolve(&context, &registry)
}
