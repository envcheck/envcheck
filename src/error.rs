//! Error types for envcheck.
//!
//! Uses `thiserror` for ergonomic error handling with automatic `Display`
//! and `Error` implementations.

use std::path::PathBuf;

use thiserror::Error;

/// Result type alias using [`EnvCheckError`].
pub type Result<T> = std::result::Result<T, EnvCheckError>;

/// All possible errors that can occur in envcheck.
#[derive(Error, Debug)]
pub enum EnvCheckError {
    /// File not found at the specified path.
    #[error("file not found: {path}")]
    FileNotFound {
        /// Path that was not found.
        path: PathBuf,
    },

    /// Failed to read file contents.
    #[error("failed to read file '{path}': {source}")]
    ReadError {
        /// Path that failed to read.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to write file contents.
    #[error("failed to write file '{path}': {source}")]
    WriteError {
        /// Path that failed to write.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse .env file.
    #[error("parse error in '{path}' at line {line}: {message}")]
    ParseError {
        /// Path of the file with the error.
        path: PathBuf,
        /// Line number where the error occurred (1-indexed).
        line: usize,
        /// Description of the parse error.
        message: String,
    },

    /// Failed to parse YAML content.
    #[error("YAML parse error in '{path}': {source}")]
    YamlError {
        /// Path of the YAML file.
        path: PathBuf,
        /// Underlying YAML parse error.
        #[source]
        source: serde_yaml::Error,
    },

    /// Invalid glob pattern.
    #[error("invalid glob pattern '{pattern}': {source}")]
    GlobError {
        /// The invalid glob pattern.
        pattern: String,
        /// Underlying glob error.
        #[source]
        source: glob::PatternError,
    },

    /// No files matched the provided pattern or arguments.
    #[error("no files matched: {pattern}")]
    NoFilesMatched {
        /// Pattern that matched nothing.
        pattern: String,
    },

    /// Invalid output format specified.
    #[error("invalid output format: '{format}'. Valid formats: text, json, github")]
    InvalidFormat {
        /// The invalid format string.
        format: String,
    },

    /// Compare command requires at least two files.
    #[error("compare requires at least two files, got {count}")]
    InsufficientFiles {
        /// Number of files provided.
        count: usize,
    },

    /// K8s sync requires --env flag.
    #[error("k8s-sync requires --env flag to specify the .env file to compare against")]
    MissingEnvFile,

    /// Lint errors were found (for exit code purposes).
    #[error("found {error_count} error(s) and {warning_count} warning(s)")]
    LintFailed {
        /// Number of errors found.
        error_count: usize,
        /// Number of warnings found.
        warning_count: usize,
    },
}

impl EnvCheckError {
    /// Creates a new file not found error.
    #[must_use]
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Creates a new read error.
    #[must_use]
    pub fn read_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    /// Creates a new write error.
    #[must_use]
    pub fn write_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::WriteError {
            path: path.into(),
            source,
        }
    }

    /// Creates a new parse error.
    #[must_use]
    pub fn parse_error(path: impl Into<PathBuf>, line: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            path: path.into(),
            line,
            message: message.into(),
        }
    }

    /// Creates a new YAML error.
    #[must_use]
    pub fn yaml_error(path: impl Into<PathBuf>, source: serde_yaml::Error) -> Self {
        Self::YamlError {
            path: path.into(),
            source,
        }
    }

    /// Returns the appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::LintFailed { error_count, .. } if *error_count > 0 => 1,
            Self::LintFailed { .. } => 0, // Warnings only
            _ => 2,                       // Other errors
        }
    }
}
