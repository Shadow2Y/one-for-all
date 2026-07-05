use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::models::Value;

pub type DynCommand = fn(&[Value]) -> std::result::Result<Value, String>;

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, DynCommand>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_func<S>(&mut self, name: S, func: DynCommand)
    where
        S: Into<String>,
    {
        self.commands.insert(name.into(), func);
    }

    pub fn execute_func<S>(&self, name: S, args: &[Value]) -> Result<Value>
    where
        S: AsRef<str>,
    {
        let func = self
            .commands
            .get(name.as_ref())
            .ok_or_else(|| anyhow!("Unknown function '{}'", name.as_ref()))?;

        func(args).map_err(anyhow::Error::msg)
    }
}
