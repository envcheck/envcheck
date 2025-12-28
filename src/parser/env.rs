use std::fs;
use std::path::PathBuf;

use crate::error::{EnvCheckError, Result};

/// Represents a single environment variable entry in a .env file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvVar {
    /// The variable key (e.g., "DATABASE_URL").
    pub key: String,
    /// The variable value.
    pub value: String,
    /// The line number in the file (1-indexed).
    pub line: usize,
    /// True if the variable is exported (starts with `export`).
    pub exported: bool,
    /// True if the key is quoted.
    pub quoted: bool,
}

/// Represents a parsed .env file.
#[derive(Debug, Clone)]
pub struct EnvFile {
    /// Path to the file.
    pub path: PathBuf,
    /// List of parsed variables.
    pub vars: Vec<EnvVar>,
    /// Raw lines of the file (for reporting context).
    pub lines: Vec<String>,
}

impl EnvFile {
    /// Parses a .env file from the given path.
    pub fn parse(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let content = fs::read_to_string(&path).map_err(|e| EnvCheckError::read_error(&path, e))?;
        Self::parse_content(path, &content)
    }

    /// Parses .env content from a string.
    pub fn parse_content(path: PathBuf, content: &str) -> Result<Self> {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let mut vars = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Handle optional 'export' prefix
            let (is_exported, content) = if let Some(stripped) = trimmed.strip_prefix("export ") {
                (true, stripped.trim())
            } else {
                (false, trimmed)
            };

            // Parse key=value
            if let Some((key, value)) = content.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                // Validate key format (simple check, full validation in lint rules)
                if key.is_empty() {
                    // Let the syntax rule handle empty keys
                    continue;
                }

                vars.push(EnvVar {
                    key,
                    value,
                    line: line_num,
                    exported: is_exported,
                    quoted: false, // TODO: Basic quoted validation if needed
                });
            } else {
                // Line has content but no equals sign - invalid syntax
                // Rules will check raw lines for this
            }
        }

        Ok(Self { path, vars, lines })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "KEY=value\n# Comment\n  \nOTHER=foo";
        let env = EnvFile::parse_content(PathBuf::from("test.env"), content).unwrap();

        assert_eq!(env.vars.len(), 2);
        assert_eq!(env.vars[0].key, "KEY");
        assert_eq!(env.vars[0].value, "value");
        assert_eq!(env.vars[0].line, 1);
        assert_eq!(env.vars[1].key, "OTHER");
        assert_eq!(env.vars[1].value, "foo");
        assert_eq!(env.vars[1].line, 4);
    }

    #[test]
    fn test_parse_export() {
        let content = "export MY_VAR=123";
        let env = EnvFile::parse_content(PathBuf::from("test.env"), content).unwrap();
        assert_eq!(env.vars[0].key, "MY_VAR");
        assert!(env.vars[0].exported);
    }
}
