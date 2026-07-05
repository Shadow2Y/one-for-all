use crate::{
    context::registry::CommandRegistry,
    models::{Value, value::LiteralValue},
};
use anyhow::{Result, bail};

pub enum TemplatePart {
    Text(String),
    Expr(Expr),
}

pub struct Template {
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Literal(LiteralValue),
    Func(String, Vec<Expr>),
}

impl Template {
    pub fn parse(input: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < input.len() {
            // Look for next expression start (! or #)
            if let Some(expr_pos) = input[pos..].find(|c| c == '!' || c == '#') {
                let expr_pos = pos + expr_pos;

                // Check if it's actually an expression
                if Self::is_valid_expr_start(&input[expr_pos..]) {
                    // Collect text before expression
                    if expr_pos > pos {
                        parts.push(TemplatePart::Text(input[pos..expr_pos].to_string()));
                    }

                    // Find expression end
                    let expr_end = Self::find_expr_end(&input[expr_pos..]);
                    let expr_str = &input[expr_pos..expr_pos + expr_end];
                    parts.push(TemplatePart::Expr(Expr::parse(expr_str)?));

                    pos = expr_pos + expr_end;
                } else {
                    pos = expr_pos + 1;
                }
            } else {
                // No more expressions, consume rest as text
                if pos < input.len() {
                    parts.push(TemplatePart::Text(input[pos..].to_string()));
                }
                break;
            }
        }

        Ok(Template { parts })
    }

    fn is_valid_expr_start(s: &str) -> bool {
        (s.starts_with('!') && s[1..].contains('('))
            || (s.starts_with('#')
                && s.len() > 1
                && s[1..].chars().next().unwrap().is_alphanumeric())
    }

    fn find_expr_end(s: &str) -> usize {
        let mut depth = 0;
        let mut found_start = false;

        for (i, ch) in s.char_indices() {
            if i == 0 {
                continue;
            }
            match ch {
                '(' => {
                    depth += 1;
                    found_start = true;
                }
                ')' => {
                    depth -= 1;
                    if found_start && depth == 0 {
                        return i + 1;
                    }
                }
                c if !c.is_alphanumeric() && c != '_' && depth == 0 && found_start => return i,
                _ => {}
            }
        }
        s.len()
    }
}

fn is_expr_start(s: &str) -> bool {
    s.starts_with('!') && s[1..].find('(').is_some()
        || s.starts_with('#')
            && s[1..]
                .chars()
                .next()
                .map_or(false, |c| c.is_alphanumeric() || c == '_')
}

fn find_expr_end(s: &str) -> usize {
    let mut depth = 0;
    let mut in_expr = false;

    for (i, ch) in s.char_indices() {
        if i == 0 {
            in_expr = true;
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && in_expr {
                    return i + 1;
                }
            }
            c if !c.is_alphanumeric() && c != '_' && depth == 0 && !in_expr => return i,
            _ => {}
        }
    }
    s.len()
}

impl Expr {
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

    pub fn resolve(&self, registry: &CommandRegistry) -> Result<Value> {
        match self {
            Expr::Var(name) => Ok(crate::context::get(name.to_string())),
            Expr::Literal(lit_val) => Ok(lit_val.into()),
            Expr::Func(name, dependents) => {
                let mut args = Vec::with_capacity(dependents.len());
                for dep in dependents {
                    args.push(dep.resolve(registry)?);
                }

                registry.execute_func(name, &args)
            }
        }
    }
}
