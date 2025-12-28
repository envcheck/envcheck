use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Configuration for envcheck, loaded from `.envcheckrc.yaml` or `.envcheckrc.toml`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Rules configuration
    pub rules: RulesConfig,

    /// Ignore patterns
    pub ignore: Vec<String>,

    /// Default output format
    pub format: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Disable specific rules by ID (e.g., ["W001", "W002"])
    pub disable: Vec<String>,

    /// Treat warnings as errors
    pub warnings_as_errors: bool,

    /// Per-rule severity overrides
    pub severity: std::collections::HashMap<String, String>,
}

impl Config {
    /// Load configuration from the current directory or parents
    pub fn load() -> Self {
        Self::load_from_path(Path::new("."))
    }

    /// Load configuration from a specific path
    pub fn load_from_path(start: &Path) -> Self {
        let mut current = start.canonicalize().ok();

        while let Some(dir) = current {
            // Try YAML first
            let yaml_path = dir.join(".envcheckrc.yaml");
            if yaml_path.exists() {
                if let Ok(content) = fs::read_to_string(&yaml_path) {
                    if let Ok(config) = serde_yaml::from_str(&content) {
                        return config;
                    }
                }
            }

            // Try TOML
            let toml_path = dir.join(".envcheckrc.toml");
            if toml_path.exists() {
                if let Ok(content) = fs::read_to_string(&toml_path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return config;
                    }
                }
            }

            // Move to parent
            current = dir.parent().map(|p| p.to_path_buf());
        }

        Self::default()
    }

    /// Check if a rule is disabled
    pub fn is_rule_disabled(&self, rule_id: &str) -> bool {
        self.rules.disable.iter().any(|r| r == rule_id)
    }

    /// Load ignore patterns from .envcheckignore file
    pub fn load_ignore_file(start: &Path) -> Vec<String> {
        let mut current = start.canonicalize().ok();

        while let Some(dir) = current {
            let ignore_path = dir.join(".envcheckignore");
            if ignore_path.exists() {
                if let Ok(content) = fs::read_to_string(&ignore_path) {
                    return content
                        .lines()
                        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
            current = dir.parent().map(|p| p.to_path_buf());
        }

        Vec::new()
    }

    /// Check if a path should be ignored based on patterns
    pub fn should_ignore(path: &Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        for pattern in patterns {
            // Simple glob matching
            if pattern.starts_with('*') {
                let suffix = &pattern[1..];
                if path_str.ends_with(suffix) || file_name.ends_with(suffix) {
                    return true;
                }
            } else if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                if path_str.starts_with(prefix) || file_name.starts_with(prefix) {
                    return true;
                }
            } else if path_str.contains(pattern) || file_name.as_ref() == pattern {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.rules.disable.is_empty());
        assert!(!config.rules.warnings_as_errors);
    }

    #[test]
    fn test_yaml_parsing() {
        let yaml = r#"
rules:
  disable:
    - W001
    - W002
  warnings_as_errors: true
ignore:
  - "*.local"
format: json
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.rules.disable, vec!["W001", "W002"]);
        assert!(config.rules.warnings_as_errors);
        assert_eq!(config.ignore, vec!["*.local"]);
        assert_eq!(config.format, Some("json".to_string()));
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
format = "sarif"

[rules]
disable = ["E001"]
warnings_as_errors = false

[[ignore]]
pattern = "*.test"
"#;
        // Note: This TOML structure doesn't match our Config exactly,
        // but demonstrates the parsing capability
        let config: Config = toml::from_str(
            r#"
format = "sarif"

[rules]
disable = ["E001"]
warnings_as_errors = false
"#,
        )
        .unwrap();
        assert_eq!(config.rules.disable, vec!["E001"]);
        assert_eq!(config.format, Some("sarif".to_string()));
    }
}
