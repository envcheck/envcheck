use crate::parser::EnvFile;
use std::fmt;
use std::path::PathBuf;

pub mod duplicate;
pub mod empty;
pub mod sort;
pub mod syntax;
pub mod whitespace;

/// Unique identifier for a lint rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleId {
    E001, // Duplicate key
    E002, // Invalid syntax
    W001, // Empty value
    W002, // Trailing whitespace
    W003, // Unsorted keys (future)
    W004, // Missing key in comparison
    W005, // K8s Secret missing in .env
    W006, // .env Key not used in K8s
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Info => write!(f, "Info"),
        }
    }
}

/// A diagnostic message produced by a rule.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub id: RuleId,
    pub severity: Severity,
    pub message: String,
    pub path: PathBuf,
    pub line: Option<usize>,
}

/// Trait implemented by all lint rules.
pub trait Rule {
    /// Returns the unique ID of the rule.
    fn id(&self) -> RuleId;

    /// Runs the rule check on the given .env file.
    fn check(&self, env_file: &EnvFile) -> Vec<Diagnostic>;
}

/// Runs all registered lint rules on a file.
#[must_use]
pub fn check_file(env_file: &EnvFile) -> Vec<Diagnostic> {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(duplicate::DuplicateKeyRule),
        Box::new(syntax::SyntaxRule),
        Box::new(empty::EmptyValueRule),
        Box::new(whitespace::TrailingWhitespaceRule),
        Box::new(sort::UnsortedKeysRule),
    ];

    let mut diagnostics = Vec::new();
    for rule in rules {
        diagnostics.extend(rule.check(env_file));
    }

    // Sort diagnostics by line number for better readability
    diagnostics.sort_by_key(|d| d.line.unwrap_or(0));

    diagnostics
}
