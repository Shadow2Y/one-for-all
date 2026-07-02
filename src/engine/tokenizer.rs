use crate::context::ExecutionContext;
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Literal(String),
    Func(String, Vec<Expr>),
}

impl Expr {
    /// Parse a custom string like "!add(!sub(#a,#b), #c)" into the AST
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();

        if let Some(var_name) = trimmed.strip_prefix('#') {
            if var_name.is_empty() {
                bail!("var_name is empty")
            }
            return Ok(Expr::Var(var_name.to_string()));
        }

        if let Some(func_content) = trimmed.strip_prefix('!') {
            let paren_pos = func_content
                .find('(')
                .ok_or_else(|| anyhow::anyhow!("Function missing opening parenthesis"))?;

            let func_name = func_content[..paren_pos].trim().to_string();

            let close_paren = func_content
                .rfind(')')
                .ok_or_else(|| anyhow::anyhow!("Function missing closing parenthesis"))?;

            let args_str = &func_content[paren_pos + 1..close_paren];

            let mut dependents = Vec::new();
            if !args_str.trim().is_empty() {
                for arg in Self::split_args(args_str) {
                    dependents.push(Self::parse(arg.trim())?);
                }
            }

            return Ok(Expr::Func(func_name, dependents));
        }
        return Ok(Expr::Literal(trimmed.to_string()));
    }

    fn split_args(args_str: &str) -> Vec<&str> {
        let mut args = Vec::new();
        let mut current_arg_start = 0;
        let mut paren_depth = 0;

        for (i, ch) in args_str.char_indices() {
            match ch {
                '(' => paren_depth += 1,
                ')' => paren_depth -= 1,
                ',' if paren_depth == 0 => {
                    args.push(&args_str[current_arg_start..i]);
                    current_arg_start = i + 1;
                }
                _ => {}
            }
        }

        if current_arg_start < args_str.len() {
            args.push(&args_str[current_arg_start..]);
        }

        args
    }

    /// Recursively evaluate the AST against the execution context
    pub fn resolve(&self, context: &ExecutionContext) -> Result<String> {
        match self {
            Expr::Var(name) => context.get_var(name),
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Func(name, dependents) => {
                let mut args = Vec::with_capacity(dependents.len());
                for dep in dependents {
                    args.push(dep.resolve(context)?);
                }
                context.execute_func(name, args)
            }
        }
    }
}
