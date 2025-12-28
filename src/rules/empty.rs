use crate::parser::EnvFile;
use crate::rules::{Diagnostic, Rule, RuleId, Severity};

pub struct EmptyValueRule;

impl Rule for EmptyValueRule {
    fn id(&self) -> RuleId {
        RuleId::W001
    }

    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for var in &env_file.vars {
            if var.value.is_empty() {
                diagnostics.push(Diagnostic {
                    id: self.id(),
                    severity: Severity::Warning,
                    message: format!("Key '{}' has an empty value", var.key),
                    path: env_file.path.clone(),
                    line: Some(var.line),
                });
            }
        }

        diagnostics
    }
}
