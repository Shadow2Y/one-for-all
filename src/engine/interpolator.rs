use super::tokenizer::{Token, tokenize};
use serde_json::Value;
use std::collections::HashMap;

/// Resolve all `#var` references in `input` using `values`.
///
/// Unknown references are silently replaced with an empty string, preserving
/// the existing test behavior. All other text is passed through unchanged.
pub fn interpolate(input: &str, values: &HashMap<String, Value>) -> String {
    let tokens = tokenize(input);
    let mut output = String::with_capacity(input.len());

    for token in tokens {
        match token {
            Token::Text(text) => output.push_str(text),

            Token::Reference { name, .. } => {
                if let Some(value) = values.get(name) {
                    match value {
                        Value::String(s) => output.push_str(s),
                        // Numbers, bools, etc. → their JSON representation
                        other => output.push_str(&other.to_string()),
                    }
                }
                // Missing reference → empty string (no-op push)
            }
        }
    }

    output
}
