use std::collections::HashMap;

use crate::parser::EnvFile;
use crate::rules::{Diagnostic, Rule, RuleId, Severity};

pub struct DuplicateKeyRule;

impl Rule for DuplicateKeyRule {
    fn id(&self) -> RuleId {
        RuleId::E001
    }

    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut seen_keys = HashMap::new();

        for var in &env_file.vars {
            if let Some(prev_line) = seen_keys.insert(&var.key, var.line) {
                diagnostics.push(Diagnostic {
                    id: self.id(),
                    severity: Severity::Error,
                    message: format!(
                        "Duplicate key '{}' (first defined on line {})",
                        var.key, prev_line
                    ),
                    path: env_file.path.clone(),
                    line: Some(var.line),
                });
            }
        }

        diagnostics
    }
}
