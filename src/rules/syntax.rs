use crate::parser::EnvFile;
use crate::rules::{Diagnostic, Rule, RuleId, Severity};

pub struct SyntaxRule;

impl Rule for SyntaxRule {
    fn id(&self) -> RuleId {
        RuleId::E002
    }

    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in env_file.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Remove optional export prefix for validation
            let content = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim();

            // Basic syntax check: must contain '=' and not start with =
            if !content.contains('=') {
                diagnostics.push(Diagnostic {
                    id: self.id(),
                    severity: Severity::Error,
                    message: format!(
                        "Invalid syntax: missing assignment operator '=' in line '{trimmed}'"
                    ),
                    path: env_file.path.clone(),
                    line: Some(line_num),
                });
            } else if content.starts_with('=') {
                diagnostics.push(Diagnostic {
                    id: self.id(),
                    severity: Severity::Error,
                    message: "Invalid syntax: key name cannot be empty".to_string(),
                    path: env_file.path.clone(),
                    line: Some(line_num),
                });
            } else {
                // Check valid key format (alphanumeric + underscore + dot/dash strictly?)
                // Generally keys are [a-zA-Z_][a-zA-Z0-9_]*
                // But some systems allow dots or dashes. Bash doesn't like dots/dashes in exports.
                // let's check for spaces in the key part
                let key_part = content.split_once('=').unwrap().0.trim();

                // "INVALID KEY=value" -> key part is "INVALID KEY"
                if key_part.contains(char::is_whitespace) {
                    diagnostics.push(Diagnostic {
                        id: self.id(),
                        severity: Severity::Error,
                        message: format!("Invalid syntax: key '{key_part}' contains whitespace"),
                        path: env_file.path.clone(),
                        line: Some(line_num),
                    });
                }

                // Check if key starts with a number (common mistake)
                if key_part.chars().next().is_some_and(char::is_numeric) {
                    diagnostics.push(Diagnostic {
                        id: self.id(),
                        severity: Severity::Error,
                        message: format!(
                            "Invalid syntax: key '{key_part}' cannot start with a number"
                        ),
                        path: env_file.path.clone(),
                        line: Some(line_num),
                    });
                }
            }
        }

        diagnostics
    }
}
