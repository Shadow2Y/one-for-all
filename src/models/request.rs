use crate::models::command::Command;

pub struct ExecutionRequest {
    pub dry_run: bool,
    pub verbose: bool,
    pub interpolate: bool,
    pub cmd: Command,
    pub name: String,
    pub args: Vec<String>,
}

impl ExecutionRequest {
    pub fn new(name: String, cmd: Command, args: Vec<String>) -> Self {
        ExecutionRequest {
            dry_run: false,
            verbose: false,
            interpolate: false,
            cmd: cmd,
            name: name,
            args: args,
        }
    }
}
