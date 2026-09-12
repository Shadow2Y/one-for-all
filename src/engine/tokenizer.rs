use std::io::{Error, ErrorKind, Result};

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

impl Template {
    pub fn parse(input: &str) -> Result<Self> {
        let mut parts = Vec::new();
        let mut pos = 0;
        let mut current_text = String::new();

        while pos < input.len() {
            let rest = &input[pos..];

            if rest.starts_with(r"\{{") {
                current_text.push_str("{{");
                pos += 3;
                continue;
            }

            if rest.starts_with("{{") {
                if !current_text.is_empty() {
                    parts.push(TemplatePart::Text(std::mem::take(&mut current_text)));
                }

                let expr_end = expr_boundary(rest)?;
                let expr_src = rest[2..expr_end - 2].trim();

                if expr_src.is_empty() {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        "Empty template expression",
                    ));
                }

                parts.push(TemplatePart::Expr(expr_src.to_string()));
                pos += expr_end;
                continue;
            }

            let ch = rest.chars().next().unwrap();
            current_text.push(ch);
            pos += ch.len_utf8();
        }

        if !current_text.is_empty() {
            parts.push(TemplatePart::Text(current_text));
        }

        Ok(Template { parts })
    }
}

/// Returns the byte length up to and including the closing `}}`.
fn expr_boundary(s: &str) -> Result<usize> {
    let Some(end) = s[2..].find("}}") else {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Unclosed template expression",
        ));
    };

    Ok(end + 4)
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
