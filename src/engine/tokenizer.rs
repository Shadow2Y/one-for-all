use std::any::{Any, TypeId};

use crate::{
    context::{ExecutionContext, registry::CommandRegistry},
    models::Value,
};
use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Literal(LiteralValue),
    Func(String, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Int(i64),
    Float(f64),
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::String(s) => write!(f, "{}", s),
            LiteralValue::Int(i) => write!(f, "{}", i),
            LiteralValue::Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl LiteralValue {
    /// Attempts to coerce a dynamically typed value into a standard literal
    pub fn from_any(any: &dyn Any) -> Option<Self> {
        let id = any.type_id();

        match id {
            // Your parser integers
            _ if id == TypeId::of::<i64>() => {
                Some(LiteralValue::Int(*any.downcast_ref::<i64>().unwrap()))
            }

            // Your native function returns
            _ if id == TypeId::of::<u32>() => {
                Some(LiteralValue::Int(*any.downcast_ref::<u32>().unwrap() as i64))
            }
            _ if id == TypeId::of::<i32>() => {
                Some(LiteralValue::Int(*any.downcast_ref::<i32>().unwrap() as i64))
            }

            // Floats and Strings
            _ if id == TypeId::of::<f64>() => {
                Some(LiteralValue::Float(*any.downcast_ref::<f64>().unwrap()))
            }
            _ if id == TypeId::of::<String>() => Some(LiteralValue::String(
                any.downcast_ref::<String>().unwrap().clone(),
            )),

            // Unprintable or unknown type
            _ => None,
        }
    }
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
        if let Ok(num) = trimmed.parse::<i64>() {
            Ok(Expr::Literal(LiteralValue::Int(num)))
        } else if let Ok(float) = trimmed.parse::<f64>() {
            Ok(Expr::Literal(LiteralValue::Float(float)))
        } else {
            Ok(Expr::Literal(LiteralValue::String(trimmed.to_string())))
        }
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

    pub fn resolve(&self, context: &ExecutionContext, registry: &CommandRegistry) -> Result<Value> {
        match self {
            Expr::Var(name) => {
                let var_str = context.get_var(name)?;
                Ok(Value::String(var_str))
            }
            Expr::Literal(lit_val) => {
                // Your parser directly maps to our main Value types now!
                match lit_val {
                    LiteralValue::Int(n) => Ok(Value::Int(*n)),
                    LiteralValue::Float(f) => Ok(Value::Float(*f)),
                    LiteralValue::String(s) => Ok(Value::String(s.clone())),
                }
            }
            Expr::Func(name, dependents) => {
                let mut args = Vec::with_capacity(dependents.len());
                for dep in dependents {
                    args.push(dep.resolve(context, registry)?);
                }

                // Pass the flat slice of values directly—no pointer-mapping loops!
                registry.execute_func(context, name, &args)
            }
        }
    }
}
