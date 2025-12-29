use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use crate::error::{EnvCheckError, Result};

/// Represents a single environment variable entry in a .env file.
/// Uses Cow<str> for zero-copy parsing when possible.
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

/// Zero-copy version of EnvVar for internal parsing
#[derive(Debug, Clone)]
pub struct EnvVarRef<'a> {
    /// The variable key (borrowed when possible).
    pub key: Cow<'a, str>,
    /// The variable value (borrowed when possible).
    pub value: Cow<'a, str>,
    /// The line number in the file (1-indexed).
    pub line: usize,
    /// True if the variable is exported.
    pub exported: bool,
}

impl<'a> EnvVarRef<'a> {
    /// Convert to owned EnvVar
    #[must_use]
    pub fn to_owned(&self) -> EnvVar {
        EnvVar {
            key: self.key.to_string(),
            value: self.value.to_string(),
            line: self.line,
            exported: self.exported,
            quoted: false,
        }
    }
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
        let vars = parse_env_zero_copy(content);

        Ok(Self { path, vars, lines })
    }
}

/// Zero-copy parser for .env content
/// Returns borrowed references where possible, owned strings only when trimming is needed
fn parse_env_zero_copy(content: &str) -> Vec<EnvVar> {
    let mut vars = Vec::new();

    for (i, line) in content.lines().enumerate() {
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

        // Parse key=value using zero-copy where possible
        if let Some((key, value)) = content.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            // Validate key format
            if key.is_empty() {
                continue;
            }

            // Remove quotes from value if present (requires allocation)
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                Cow::Owned(value[1..value.len() - 1].to_string())
            } else {
                Cow::Borrowed(value)
            };

            vars.push(EnvVar {
                key: key.to_string(),      // Key always needs owned for downstream use
                value: value.into_owned(), // Convert to owned for storage
                line: line_num,
                exported: is_exported,
                quoted: false,
            });
        }
    }

    vars
}

/// High-performance iterator for parsing that avoids allocations
pub struct EnvVarIter<'a> {
    lines: std::str::Lines<'a>,
    line_num: usize,
}

impl<'a> EnvVarIter<'a> {
    #[must_use]
    pub fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines(),
            line_num: 0,
        }
    }
}

impl<'a> Iterator for EnvVarIter<'a> {
    type Item = EnvVarRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = self.lines.next()?;
            self.line_num += 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (is_exported, content) = if let Some(stripped) = trimmed.strip_prefix("export ") {
                (true, stripped.trim())
            } else {
                (false, trimmed)
            };

            if let Some((key, value)) = content.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                if key.is_empty() {
                    continue;
                }

                return Some(EnvVarRef {
                    key: Cow::Borrowed(key),
                    value: Cow::Borrowed(value),
                    line: self.line_num,
                    exported: is_exported,
                });
            }
        }
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

    #[test]
    fn test_zero_copy_iterator() {
        let content = "KEY=value\nOTHER=foo";
        let vars: Vec<_> = EnvVarIter::new(content).collect();

        assert_eq!(vars.len(), 2);
        assert!(matches!(vars[0].key, Cow::Borrowed(_)));
        assert!(matches!(vars[0].value, Cow::Borrowed(_)));
    }

    #[test]
    fn test_quoted_values() {
        let content = "KEY=\"quoted value\"";
        let env = EnvFile::parse_content(PathBuf::from("test.env"), content).unwrap();
        assert_eq!(env.vars[0].value, "quoted value");
    }
}
