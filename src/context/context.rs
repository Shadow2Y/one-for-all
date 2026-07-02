use std::{collections::HashMap, sync::RwLock};

use anyhow::Result;

type CommandFn = Box<dyn Fn(&ExecutionContext, Vec<String>) -> Result<String> + Send + Sync>;

pub struct ExecutionContext {
    variables: RwLock<HashMap<String, String>>,
    functions: HashMap<String, CommandFn>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: RwLock::new(HashMap::new()),
            functions: HashMap::new(),
        }
    }

    pub fn set_var(&self, name: impl Into<String>, value: impl Into<String>) {
        self.variables
            .write()
            .unwrap()
            .insert(name.into(), value.into());
    }

    pub fn get_var(&self, name: &str) -> Result<String> {
        self.variables
            .read()
            .unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Variable '{}' not found", name))
    }

    pub fn register_func<F>(&mut self, name: impl Into<String>, func: F)
    where
        F: Fn(&ExecutionContext, Vec<String>) -> Result<String> + Send + Sync + 'static,
    {
        self.functions.insert(name.into(), Box::new(func));
    }

    pub fn execute_func(&self, name: &str, args: Vec<String>) -> Result<String> {
        let func = self
            .functions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not registered", name))?;

        func(self, args)
    }
}
