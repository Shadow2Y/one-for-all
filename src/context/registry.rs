use anyhow::Result;
use std::collections::HashMap;

use crate::{context::ExecutionContext, models::Value};

// Fix: CommandFn now returns a unified anyhow::Result<Value>
pub type CommandFn = Box<dyn Fn(&ExecutionContext, &[Value]) -> Result<Value> + Send + Sync>;

pub struct CommandRegistry {
    functions: HashMap<String, CommandFn>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a context-aware function.
    /// Use this when the function needs to read/write variables from the ExecutionContext.
    pub fn register_context_func<F>(&mut self, name: impl Into<String>, func: F)
    where
        F: Fn(&ExecutionContext, &[Value]) -> std::result::Result<Value, String>
            + Send
            + Sync
            + 'static,
    {
        self.functions.insert(
            name.into(),
            // Fix: Boxed the closure and mapped the String error to anyhow::Error
            Box::new(move |ctx, args| func(ctx, args).map_err(|e| anyhow::anyhow!(e))),
        );
    }

    /// Register a context-free function.
    pub fn register_func(
        &mut self,
        name: impl Into<String>,
        func: fn(&[Value]) -> std::result::Result<Value, String>,
    ) {
        self.functions.insert(
            name.into(),
            Box::new(move |_ctx, args| func(args).map_err(|e| anyhow::anyhow!(e))),
        );
    }

    /// Execute a function by passing the active context and arguments into it.
    pub fn execute_func(
        &self,
        ctx: &ExecutionContext,
        name: &str,
        args: &[Value],
    ) -> Result<Value> {
        let func = self
            .functions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not registered", name))?;

        // Execute the function directly
        func(ctx, args)
    }
}
