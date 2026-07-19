use anyhow::{Result, bail};

use crate::models::value::LiteralValue;

// ── Public types ──────────────────────────────────────────────────────────────

/// A parsed template string, broken into literal text segments and embedded
/// expressions.  Resolution is intentionally *not* done here — the tokenizer
/// only produces the AST; see [`crate::engine::resolver`] for evaluation.
pub struct Template {
    pub parts: Vec<TemplatePart>,
}

pub enum TemplatePart {
    Text(String),
    Expr(Expr),
}

/// A parsed expression that can appear inside a template.
///
/// Syntax:
/// - `#name`          → [`Expr::Var`]
/// - `!func(a, b)`    → [`Expr::Func`]
/// - `42` / `3.14` / `hello` (bare literal in a function arg) → [`Expr::Literal`]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Literal(LiteralValue),
    Func(String, Vec<Expr>),
}

// ── Template parsing ──────────────────────────────────────────────────────────

impl Template {
    pub fn parse(input: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut pos = 0;

        while pos < input.len() {
            // Scan for the next expression starter ('!' or '#').
            if let Some(rel) = input[pos..].find(|c| c == '!' || c == '#') {
                let expr_pos = pos + rel;

                if is_expr_start(&input[expr_pos..]) {
                    // Flush any literal text that came before.
                    if expr_pos > pos {
                        parts.push(TemplatePart::Text(input[pos..expr_pos].to_string()));
                    }

                    let expr_end = expr_boundary(&input[expr_pos..]);
                    let expr_src = &input[expr_pos..expr_pos + expr_end];
                    parts.push(TemplatePart::Expr(Expr::parse(expr_src)?));

                    pos = expr_pos + expr_end;
                } else {
                    // The character is not actually an expression start; skip it.
                    pos = expr_pos + 1;
                }
            } else {
                // No more expressions — flush the rest as plain text.
                parts.push(TemplatePart::Text(input[pos..].to_string()));
                break;
            }
        }

        Ok(Template { parts })
    }
}

// ── Expression parsing ────────────────────────────────────────────────────────

impl Expr {
    /// Parses a single expression token.
    ///
    /// - `#name`       → `Expr::Var("name")`
    /// - `!fn(…)`      → `Expr::Func("fn", […])`
    /// - integer / float / anything else → `Expr::Literal`
    pub fn parse(input: &str) -> Result<Self> {
        let s = input.trim();

        if let Some(name) = s.strip_prefix('#') {
            if name.is_empty() {
                bail!("Variable name after '#' must not be empty");
            }
            return Ok(Expr::Var(name.to_string()));
        }

        if let Some(body) = s.strip_prefix('!') {
            let paren = body
                .find('(')
                .ok_or_else(|| anyhow::anyhow!("Function expression is missing '('"))?;

            let func_name = body[..paren].trim().to_string();

            let close = body
                .rfind(')')
                .ok_or_else(|| anyhow::anyhow!("Function expression is missing ')'"))?;

            let args_src = &body[paren + 1..close];
            let arg_exprs: Result<Vec<Expr>> = split_args(args_src)
                .into_iter()
                .filter(|a| !a.trim().is_empty())
                .map(|a| Expr::parse(a.trim()))
                .collect();

            return Ok(Expr::Func(func_name, arg_exprs?));
        }

        // Bare literal (appears as a function argument)
        if let Ok(n) = s.parse::<i64>() {
            return Ok(Expr::Literal(LiteralValue::Int(n)));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Expr::Literal(LiteralValue::Float(f)));
        }
        Ok(Expr::Literal(LiteralValue::String(s.to_string())))
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Returns `true` if `s` starts with a syntactically valid expression opener.
fn is_expr_start(s: &str) -> bool {
    (s.starts_with('!') && s[1..].contains('('))
        || (s.starts_with('#') && s[1..].chars().next().map_or(false, |c| c.is_alphanumeric()))
}

/// Returns the byte length of the expression starting at the beginning of `s`.
///
/// - Functions (`!fn(…)`) end after the matching closing `)`.
/// - Variables (`#name`) end at the first non-alphanumeric / non-`_` character.
fn expr_boundary(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }

    if s.starts_with('!') {
        // Track paren depth to find the matching close.
        let mut depth: usize = 0;
        let mut opened = false;
        for (i, ch) in s.char_indices().skip(1) {
            match ch {
                '(' => {
                    depth += 1;
                    opened = true;
                }
                ')' if depth > 0 => {
                    depth -= 1;
                    if depth == 0 && opened {
                        return i + 1;
                    }
                }
                _ => {}
            }
        }
    } else {
        // Variable: stop at first char that isn't alphanumeric or '_'.
        for (i, ch) in s.char_indices().skip(1) {
            if !ch.is_alphanumeric() && ch != '_' {
                return i;
            }
        }
    }

    s.len()
}

/// Splits a comma-separated argument string, respecting nested parentheses.
fn split_args(args: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth: usize = 0;

    for (i, ch) in args.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                result.push(&args[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < args.len() {
        result.push(&args[start..]);
    }

    result
}
