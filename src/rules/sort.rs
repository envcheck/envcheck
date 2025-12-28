use crate::parser::EnvFile;
use crate::rules::{Diagnostic, Rule, RuleId, Severity};

pub struct UnsortedKeysRule;

impl Rule for UnsortedKeysRule {
    fn id(&self) -> RuleId {
        RuleId::W003
    }

    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut last_key: Option<&str> = None;

        for var in &env_file.vars {
            if let Some(prev) = last_key {
                if var.key.as_str() < prev {
                    diagnostics.push(Diagnostic {
                        id: self.id(),
                        severity: Severity::Warning,
                        message: format!(
                            "Unsorted key '{}' should come before '{}'",
                            var.key, prev
                        ),
                        path: env_file.path.clone(),
                        line: Some(var.line),
                    });
                }
            }
            last_key = Some(&var.key);
        }

        diagnostics
    }
}
