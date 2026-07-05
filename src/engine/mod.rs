use anyhow::Result;

use crate::{context, engine::executor::execute_command, models::Value};

mod executor;
pub mod tokenizer;

pub fn handle_command(cmd: &str, args: &[String]) -> Result<Value> {
    let final_cmd = find_base_command(cmd, args);
    execute_command(context::get_registry(), &final_cmd)
}

fn find_base_command(cmd: &str, args: &[String]) -> String {
    log::debug!("{} :: {:?}", cmd, args);
    cmd.to_string()
}
