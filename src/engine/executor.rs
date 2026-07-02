use anyhow::Result;

use crate::{context::ExecutionContext, engine::tokenizer::Expr};

pub fn execute_command(context: &'static ExecutionContext, args: &[String]) -> Result<String> {
    println!("{:?}", std::env::args().collect::<Vec<_>>());
    let ast = Expr::parse(&args[0]).expect("Failed to parse expression");

    ast.resolve(&context)
}
