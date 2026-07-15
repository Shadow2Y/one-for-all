use anyhow::Result;

use crate::{
    config, context,
    models::{Value, command::Command},
};

mod executor;
pub mod tokenizer;
pub use executor::execute_command;

pub fn handle_command(cmd: &str, args: &[String]) -> Result<Value> {
    let final_cmd = find_base_command(cmd, args);
    execute_command(context::get_registry(), final_cmd.0, final_cmd.1)
}

pub fn find_base_command<'a>(cmd: &str, args: &'a [String]) -> (&'static Command, &'a [String]) {
    let config = config::get();

    let mut current = config
        .commands
        .get(cmd)
        .unwrap_or_else(|| panic!("Unknown command '{}'", cmd));

    let mut index = 0;

    while let Command::Group(children) = current {
        if index >= args.len() {
            break;
        }

        match children.get(&args[index]) {
            Some(next) => {
                current = next;
                index += 1;
            }
            None => break,
        }
    }

    (current, &args[index..])
}
