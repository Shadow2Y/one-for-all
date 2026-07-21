use anyhow::{Result, bail};

use crate::models::value::LiteralValue;

// ── Public types ──────────────────────────────────────────────────────────────

/// A parsed template string, broken into literal text segments and embedded
/// expressions. Resolution is intentionally *not* done here — the tokenizer
/// only produces the AST; see [`crate::engine::resolver`] for evaluation.
pub struct Template {
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expr(Expr),
}

/// A parsed expression that can appear inside a template.
///
/// Syntax:
/// - `@name`          → [`Expr::Var`]
/// - `@func(a, b)`    → [`Expr::Func`]
/// - `42` / `3.14` / `"hello"` (literal in function args) → [`Expr::Literal`]
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
        let mut current_text = String::new();

        while pos < input.len() {
            let rest = &input[pos..];

            // Check for escape sequence '\@' or '\\'
            if rest.starts_with("\\@") || rest.starts_with("\\\\") {
                current_text.push(rest.chars().nth(1).unwrap());
                pos += 2;
                continue;
            }

            if rest.starts_with('@') && is_expr_start(rest) {
                // Flush accumulated text if any
                if !current_text.is_empty() {
                    parts.push(TemplatePart::Text(std::mem::take(&mut current_text)));
                }

                let expr_end = expr_boundary(rest);
                let expr_src = &rest[..expr_end];
                parts.push(TemplatePart::Expr(Expr::parse(expr_src)?));

                pos += expr_end;
            } else {
                let ch = rest.chars().next().unwrap();
                current_text.push(ch);
                pos += ch.len_utf8();
            }
        }

        if !current_text.is_empty() {
            parts.push(TemplatePart::Text(current_text));
        }

        Ok(Template { parts })
    }
}

// ── Expression parsing ────────────────────────────────────────────────────────

impl Expr {
    /// Parses a single expression token starting with `@` (or bare literal).
    ///
    /// - `@name`       → `Expr::Var("name")`
    /// - `@fn(…)`      → `Expr::Func("fn", […])`
    /// - integer / float / quoted string / bare string → `Expr::Literal`
    pub fn parse(input: &str) -> Result<Self> {
        let s = input.trim();

        if let Some(body) = s.strip_prefix('@') {
            if body.is_empty() {
                bail!("Variable or function name after '@' must not be empty");
            }

            // Check if function call (contains '(' after name)
            if let Some(paren) = body.find('(') {
                let func_name = body[..paren].trim().to_string();
                if func_name.is_empty() {
                    bail!("Function name cannot be empty");
                }
                let close = body
                    .rfind(')')
                    .ok_or_else(|| anyhow::anyhow!("Function expression missing ')'"))?;
                let args_src = &body[paren + 1..close];

                let arg_exprs: Result<Vec<Expr>> = split_args(args_src)
                    .into_iter()
                    .filter(|a| !a.trim().is_empty())
                    .map(|a| Expr::parse(a.trim()))
                    .collect();

                return Ok(Expr::Func(func_name, arg_exprs?));
            }

            return Ok(Expr::Var(body.to_string()));
        }

        // Quoted string literal unquoting ("hello" or 'hello')
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            let inner = &s[1..s.len() - 1];
            let unescaped = inner.replace("\\\"", "\"").replace("\\\\", "\\");
            return Ok(Expr::Literal(LiteralValue::String(unescaped)));
        }
        if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
            let inner = &s[1..s.len() - 1];
            let unescaped = inner.replace("\\'", "'").replace("\\\\", "\\");
            return Ok(Expr::Literal(LiteralValue::String(unescaped)));
        }

        // Bare number or string literal (appears inside function arguments)
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

/// Returns `true` if `s` starts with `@` followed by an alphanumeric character or `_`.
fn is_expr_start(s: &str) -> bool {
    let mut chars = s.chars();
    if chars.next() == Some('@') {
        if let Some(next) = chars.next() {
            return next.is_alphanumeric() || next == '_';
        }
    }
    false
}

/// Returns the byte length of the expression starting at the beginning of `s`.
///
/// - Functions (`@fn(…)`) end after the matching closing `)`.
/// - Variables (`@name`) end at the first non-alphanumeric / non-`_` character.
fn expr_boundary(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }

    // Check if it's a function invocation `@fn(...)`
    if let Some(paren_pos) = s.find('(') {
        let name_part = &s[..paren_pos];
        if name_part.chars().skip(1).all(|c| c.is_alphanumeric() || c == '_') {
            let mut depth: usize = 0;
            let mut in_quote = None;
            let mut escaped = false;

            for (i, ch) in s.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }

                match ch {
                    '"' | '\'' => match in_quote {
                        Some(q) if q == ch => in_quote = None,
                        None => in_quote = Some(ch),
                        _ => {}
                    },
                    '(' if in_quote.is_none() => depth += 1,
                    ')' if in_quote.is_none() => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return i + 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Variable: stop at first char that isn't alphanumeric or '_'.
    for (i, ch) in s.char_indices().skip(1) {
        if !ch.is_alphanumeric() && ch != '_' {
            return i;
        }
    }

    s.len()
}

/// Splits a comma-separated argument string, respecting nested parentheses and quotes.
fn split_args(args: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth: usize = 0;
    let mut in_quote = None;
    let mut escaped = false;

    for (i, ch) in args.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }

        match ch {
            '"' | '\'' => match in_quote {
                Some(q) if q == ch => in_quote = None,
                None => in_quote = Some(ch),
                _ => {}
            },
            '(' if in_quote.is_none() => depth += 1,
            ')' if in_quote.is_none() => depth = depth.saturating_sub(1),
            ',' if depth == 0 && in_quote.is_none() => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_parsing() {
        let tmpl = Template::parse("java -jar @jar_path --opt=@_opt1").unwrap();
        assert_eq!(tmpl.parts.len(), 4);
        match &tmpl.parts[1] {
            TemplatePart::Expr(Expr::Var(name)) => assert_eq!(name, "jar_path"),
            _ => panic!("Expected Expr::Var(jar_path)"),
        }
        match &tmpl.parts[3] {
            TemplatePart::Expr(Expr::Var(name)) => assert_eq!(name, "_opt1"),
            _ => panic!("Expected Expr::Var(_opt1)"),
        }
    }

    #[test]
    fn test_function_parsing_with_quotes() {
        let tmpl = Template::parse("echo @concat(@var, \"hello, world\")").unwrap();
        assert_eq!(tmpl.parts.len(), 2);
        match &tmpl.parts[1] {
            TemplatePart::Expr(Expr::Func(name, args)) => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], Expr::Var("var".to_string()));
                assert_eq!(
                    args[1],
                    Expr::Literal(LiteralValue::String("hello, world".to_string()))
                );
            }
            _ => panic!("Expected Expr::Func"),
        }
    }

    #[test]
    fn test_escaped_at_sign() {
        let tmpl = Template::parse("user\\@example.com @var").unwrap();
        assert_eq!(tmpl.parts.len(), 2);
        match &tmpl.parts[0] {
            TemplatePart::Text(t) => assert_eq!(t, "user@example.com "),
            _ => panic!("Expected Text"),
        }
        match &tmpl.parts[1] {
            TemplatePart::Expr(Expr::Var(name)) => assert_eq!(name, "var"),
            _ => panic!("Expected Expr::Var"),
        }
    }

    #[test]
    fn test_function_parsing_with_escaped_quotes() {
        let tmpl = Template::parse("echo @concat(\"a, \\\"b, c\\\"\", \"d\")").unwrap();
        assert_eq!(tmpl.parts.len(), 2);
        match &tmpl.parts[1] {
            TemplatePart::Expr(Expr::Func(name, args)) => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 2);
                assert_eq!(
                    args[0],
                    Expr::Literal(LiteralValue::String("a, \"b, c\"".to_string()))
                );
                assert_eq!(
                    args[1],
                    Expr::Literal(LiteralValue::String("d".to_string()))
                );
            }
            _ => panic!("Expected Expr::Func"),
        }
    }

    #[test]
    fn test_function_parsing_with_single_quotes() {
        let tmpl = Template::parse("echo @concat('a, \\'b, c\\'', '')").unwrap();
        assert_eq!(tmpl.parts.len(), 2);
        match &tmpl.parts[1] {
            TemplatePart::Expr(Expr::Func(name, args)) => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 2);
                assert_eq!(
                    args[0],
                    Expr::Literal(LiteralValue::String("a, 'b, c'".to_string()))
                );
                assert_eq!(
                    args[1],
                    Expr::Literal(LiteralValue::String("".to_string()))
                );
            }
            _ => panic!("Expected Expr::Func"),
        }
    }

    #[test]
    fn test_non_expr_at_sign() {
        let tmpl = Template::parse("contact user@ domain").unwrap();
        assert_eq!(tmpl.parts.len(), 1);
        match &tmpl.parts[0] {
            TemplatePart::Text(t) => assert_eq!(t, "contact user@ domain"),
            _ => panic!("Expected Text"),
        }
    }
}



