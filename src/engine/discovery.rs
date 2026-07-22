use std::collections::HashMap;

use crate::{
    config,
    models::{
        Value,
        command::{Command, CommandKind, ExecutionMode},
    },
};

/// Formats and lists all available base commands from resolved configuration.
pub fn list_base_commands() -> Value {
    let cfg = config::get();
    let mut out = String::new();

    out.push_str("ofa - The one CLI tool to orchestrate them all\n");

    if let Some(ref profile) = cfg.profile {
        out.push_str(&format!("\nActive Profile: {}\n", profile));
    } else {
        out.push_str("\nActive Profile: none\n");
    }

    if let Some(local_path) = config::find_local_config() {
        out.push_str(&format!("Local Config: {}\n", local_path.display()));
    }

    out.push_str("\nAvailable Commands:\n");

    let mut keys: Vec<&String> = cfg.commands.keys().collect();
    keys.sort();

    if keys.is_empty() {
        out.push_str("  (no commands configured)\n");
    } else {
        for key in keys {
            if let Some(cmd) = cfg.commands.get(key) {
                out.push_str(&format!("  {:20} {}\n", key, format_cmd_type(cmd)));
            }
        }
    }

    // Include built-in app command
    out.push_str(&format!("  {:20} [builtin]\n", "app"));

    out.push_str("\nUsage:\n");
    out.push_str("  ofa <command> [args...]\n");

    Value::String(out)
}

/// Formats available subcommands for a command group.
pub fn list_group_subcommands(group_path: &str, children: &HashMap<String, Command>) -> Value {
    let mut out = String::new();

    out.push_str(&format!("'{}' is a command group.\n\n", group_path));
    out.push_str(&format!("Available Subcommands for 'ofa {}':\n", group_path));

    let mut keys: Vec<&String> = children.keys().collect();
    keys.sort();

    if keys.is_empty() {
        out.push_str("  (no subcommands in group)\n");
    } else {
        for key in keys {
            if let Some(cmd) = children.get(key) {
                out.push_str(&format!("  {:20} {}\n", key, format_cmd_type(cmd)));
            }
        }
    }

    out.push_str("\nUsage:\n");
    out.push_str(&format!("  ofa {} <subcommand> [args...]\n", group_path));

    Value::String(out)
}

fn format_cmd_type(cmd: &Command) -> String {
    match &cmd.cmd {
        CommandKind::Group(children) => format!("[group] ({} subcommands)", children.len()),
        _ => match cmd.kind {
            ExecutionMode::Shell => "[shell]".to_string(),
            ExecutionMode::Template => "[template]".to_string(),
            ExecutionMode::TemplateShell => "[template_shell]".to_string(),
        },
    }
}
