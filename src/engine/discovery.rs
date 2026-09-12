use std::collections::HashMap;

use crate::models::command::{Command, CommandKind, ExecutionMode};

pub fn list_group_subcommands(group_path: &str, children: &HashMap<String, Command>) -> String {
    let mut out = String::new();

    out.push_str(&format!("'{}' is a command group.\n\n", group_path));
    out.push_str("Commands:\n");

    let mut keys: Vec<&String> = children.keys().collect();
    keys.sort();

    if keys.is_empty() {
        out.push_str("  (no commands configured)\n");
    } else {
        for key in keys {
            if let Some(cmd) = children.get(key) {
                out.push_str(&format!("  {:10} {}\n", key, format_cmd_type(cmd)));
            }
        }
    }

    out.push_str(&format!(
        "\nUsage: ofa {} <command> [args...]\n",
        group_path
    ));

    out.push_str(&format!(
        "\nRun 'ofa {} <command> --help' for command-specific help.\n",
        group_path
    ));

    out
}

pub fn format_cmd_type(cmd: &Command) -> String {
    match &cmd.cmd {
        CommandKind::Group(children) => {
            format!("[group] ({} subcommands)", children.len())
        }
        _ => match cmd.kind {
            ExecutionMode::Shell => "[shell]".to_string(),
            ExecutionMode::TemplateShell => "[template_shell]".to_string(),
        },
    }
}
