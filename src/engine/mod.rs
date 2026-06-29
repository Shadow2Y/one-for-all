use anyhow::Result;
use log::info;

mod executor;
mod interpolator;
pub mod tokenizer;

// ── Public API ────────────────────────────────────────────────────────────────

/// Dispatch a user command (e.g. `ofa build`) through the resolved config.
///
/// Lookup order: `commands` map first, then `runnables`. This lets type-level
/// commands be shadowed by local runnables if needed.
pub fn handle_command(command: &str, args: Vec<String>) -> Result<()> {
    info!("Running command '{}' with args {:?}", command, args);

    let config = crate::config::get_config()?;

    let runnable = config
        .commands
        .get(command)
        .or_else(|| config.runnables.get(command))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown command '{}'. Run `ofa` to see available commands.",
                command
            )
        })?;

    executor::execute_user(runnable, &args, &config.vars)
}

/// Fire a lifecycle hook (e.g. `pre-build`, `post-test`) if one is registered.
/// Silently no-ops if the lifecycle has no handler.
pub fn handle_lifecycle(lifecycle: &str, args: Vec<String>) -> Result<()> {
    info!("Lifecycle '{}' with args {:?}", lifecycle, args);

    let config = crate::config::get_config()?;

    if let Some(runnable) = config.lifecycle.get(lifecycle) {
        executor::execute_user(runnable, &args, &config.vars)?;
    } else {
        log::debug!("No lifecycle handler for '{}'", lifecycle);
    }

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use interpolator::interpolate;
    use serde_json::Value;
    use std::collections::HashMap;
    use tokenizer::{Token, tokenize};

    #[test]
    fn tokenize_plain_text() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec![Token::Text("hello world")]);
    }

    #[test]
    fn tokenize_single_reference() {
        let tokens = tokenize("#key");
        assert_eq!(
            tokens,
            vec![Token::Reference {
                name: "key",
                span: 0..4,
            }]
        );
    }

    #[test]
    fn tokenize_mixed_text_and_references() {
        let tokens = tokenize("mkdir -p #data_dir && echo #value");
        assert_eq!(
            tokens,
            vec![
                Token::Text("mkdir -p "),
                Token::Reference {
                    name: "data_dir",
                    span: 9..18,
                },
                Token::Text(" && echo "),
                Token::Reference {
                    name: "value",
                    span: 27..33,
                },
            ]
        );
    }

    #[test]
    fn tokenize_adjacent_references() {
        let tokens = tokenize("#dir/#file");
        assert_eq!(
            tokens,
            vec![
                Token::Reference {
                    name: "dir",
                    span: 0..4,
                },
                Token::Text("/"),
                Token::Reference {
                    name: "file",
                    span: 5..10,
                },
            ]
        );
    }

    #[test]
    fn tokenize_invalid_hash_is_text() {
        let tokens = tokenize("# #1 #-");
        assert_eq!(
            tokens,
            vec![
                Token::Text("#"),
                Token::Text(" "),
                Token::Text("#"),
                Token::Text("1 "),
                Token::Text("#"),
                Token::Text("-"),
            ]
        );
    }

    #[test]
    fn tokenize_underscore_reference() {
        let tokens = tokenize("#data_dir");
        assert_eq!(
            tokens,
            vec![Token::Reference {
                name: "data_dir",
                span: 0..9,
            }]
        );
    }

    #[test]
    fn interpolate_single_value() {
        let mut vars = HashMap::new();
        vars.insert("branch".into(), Value::String("main".into()));
        let result = interpolate("Current branch: #branch", &vars);
        assert_eq!(result, "Current branch: main");
    }

    #[test]
    fn interpolate_multiple_values() {
        let mut vars = HashMap::new();
        vars.insert("data_dir".into(), Value::String("/tmp/state".into()));
        vars.insert("key".into(), Value::String("branch".into()));
        let result = interpolate("#data_dir/#key", &vars);
        assert_eq!(result, "/tmp/state/branch");
    }

    #[test]
    fn interpolate_json_number() {
        let mut vars = HashMap::new();
        vars.insert("count".into(), Value::Number(42.into()));
        let result = interpolate("count=#count", &vars);
        assert_eq!(result, "count=42");
    }

    #[test]
    fn interpolate_missing_value() {
        let vars = HashMap::new();
        let result = interpolate("branch=#branch", &vars);
        assert_eq!(result, "branch=");
    }

    #[test]
    fn interpolate_real_command() {
        let mut vars = HashMap::new();
        vars.insert("data_dir".into(), Value::String("/tmp/data".into()));
        vars.insert("key".into(), Value::String("branch".into()));
        vars.insert("value".into(), Value::String("main".into()));
        let result = interpolate(
            "mkdir -p #data_dir && printf '%s' \"#value\" > \"#data_dir/#key\"",
            &vars,
        );
        assert_eq!(
            result,
            "mkdir -p /tmp/data && printf '%s' \"main\" > \"/tmp/data/branch\""
        );
    }
}
