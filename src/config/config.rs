use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::{command::Command, variable::Variable};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profile: Option<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub vars: HashMap<String, Variable>,

    #[serde(default)]
    pub commands: HashMap<String, Command>,
}

impl Config {
    /// Overlays `other` config on top of `self`.
    /// Key-value maps (`env`, `vars`, `commands`) are overwritten by matching keys in `other`.
    pub fn merge(&mut self, other: Config) {
        if other.profile.is_some() {
            self.profile = other.profile;
        }
        for (k, v) in other.env {
            self.env.insert(k, v);
        }
        for (k, v) in other.vars {
            self.vars.insert(k, v);
        }
        for (k, v) in other.commands {
            self.commands.insert(k, v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_merge_overrides_keys() {
        let mut global = Config::default();
        global.env.insert("ENV_A".to_string(), "global_val".to_string());
        global.env.insert("ENV_B".to_string(), "global_val".to_string());

        let mut local = Config::default();
        local.profile = Some("java".to_string());
        local.env.insert("ENV_B".to_string(), "local_override".to_string());
        local.env.insert("ENV_C".to_string(), "local_val".to_string());

        global.merge(local);

        assert_eq!(global.profile, Some("java".to_string()));
        assert_eq!(global.env.get("ENV_A").unwrap(), "global_val");
        assert_eq!(global.env.get("ENV_B").unwrap(), "local_override");
        assert_eq!(global.env.get("ENV_C").unwrap(), "local_val");
    }
}


