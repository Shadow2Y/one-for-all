// ── Public types ──────────────────────────────────────────────────────────────

use std::io::Error;

/// A parsed template string, broken into literal text segments and embedded
/// expressions. Resolution is intentionally *not* done here — the tokenizer
/// only produces the AST; see [`crate::engine::resolver`] for evaluation.
pub struct Template {
    pub parts: Vec<TemplatePart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Expr(String),
}

// ── Template parsing ──────────────────────────────────────────────────────────

impl Template {
    pub fn parse(input: &str) -> Result<Self, Error> {
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
                let expr_src = &rest[1..expr_end];
                parts.push(TemplatePart::Expr(expr_src.to_string()));

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

    // Variable: stop at first char that isn't alphanumeric or '_'.
    for (i, ch) in s.char_indices().skip(1) {
        if !ch.is_alphanumeric() && ch != '_' {
            return i;
        }
    }

    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_parsing() {
        let tmpl = Template::parse("java -jar @jar_path --opt=@_opt1").unwrap();
        assert_eq!(tmpl.parts.len(), 4);
        match &tmpl.parts[1] {
            TemplatePart::Expr(name) => assert_eq!(name, "jar_path"),
            _ => panic!("Expected Expr::Var(jar_path)"),
        }
        match &tmpl.parts[3] {
            TemplatePart::Expr(name) => assert_eq!(name, "_opt1"),
            _ => panic!("Expected Expr::Var(_opt1)"),
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
            TemplatePart::Expr(name) => assert_eq!(name, "var"),
            _ => panic!("Expected Expr::Var"),
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
