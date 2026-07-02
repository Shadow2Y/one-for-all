use anyhow::Result;

use crate::{config, context, engine::executor::execute_command};

mod executor;
pub mod tokenizer;

pub fn handle_command(cmd: &str, args: &[String]) -> Result<String> {
    find_base_command(cmd, args);
    context::get();
    config::get();
    execute_command(context::get(), args)
}

fn find_base_command(cmd: &str, args: &[String]) -> String {
    print!("{} :: {:?}", cmd, args);
    cmd.to_string()
}
