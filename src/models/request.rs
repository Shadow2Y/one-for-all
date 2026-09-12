use crate::models::command::Command;

pub struct ExecutionRequest {
    pub dry_run: bool,
    pub verbose: bool,
    pub interpolate: bool,
    pub cmd: Command,
    pub args: Vec<String>,
}

impl ExecutionRequest {
    pub fn new(cmd: Command, args: Vec<String>) -> Self {
        ExecutionRequest {
            dry_run: false,
            verbose: false,
            interpolate: false,
            cmd: cmd,
            args: args,
        }
    }
}
