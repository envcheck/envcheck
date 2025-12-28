use crate::parser::EnvFile;
use crate::rules::{Diagnostic, Rule, RuleId, Severity};

pub struct TrailingWhitespaceRule;

impl Rule for TrailingWhitespaceRule {
    fn id(&self) -> RuleId {
        RuleId::W002
    }

    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in env_file.lines.iter().enumerate() {
            let line_num = i + 1;

            // Allow trailing whitespace in comments? Usually yes, but here let's be strict if the rule implies strictness.
            // But stripping whitespace is standard behavior for parsers.
            // The requirement says "Trailing whitespace" warning.

            if line.len() != line.trim_end().len() {
                diagnostics.push(Diagnostic {
                    id: self.id(),
                    severity: Severity::Warning,
                    message: "Line contains trailing whitespace".to_string(),
                    path: env_file.path.clone(),
                    line: Some(line_num),
                });
            }
        }

        diagnostics
    }
}
