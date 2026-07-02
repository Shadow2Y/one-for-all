use anyhow::Result;

use crate::{context, engine::executor::execute_command, models::Value};

mod executor;
pub mod tokenizer;

pub fn handle_command(cmd: &str, args: &[String]) -> Result<Value> {
    execute_command(context::get_context(), context::get_registry(), args)
}

fn find_base_command(cmd: &str, args: &[String]) -> String {
    print!("{} :: {:?}", cmd, args);
    cmd.to_string()
}
