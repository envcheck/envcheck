//! envcheck - A fast, modern CLI for linting .env files and detecting K8s ↔ env mismatches.
//!
//! # Features
//!
//! - **Lint** `.env` files for syntax errors, duplicate keys, and common issues
//! - **Compare** environment files across environments (local, staging, prod)
//! - **K8s Sync** detection between Kubernetes manifests and .env files
//!
//! # Example
//!
//! ```rust,ignore
//! use envcheck::{EnvFile, lint};
//!
//! let env_file = EnvFile::parse("DATABASE_URL=postgres://localhost/db")?;
//! let diagnostics = lint(&env_file);
//! ```

#![doc(html_root_url = "https://docs.rs/envcheck/0.1.0")]

pub mod commands;
pub mod error;
pub mod output;
pub mod parser;
pub mod rules;

// Re-export main types for convenience
pub use error::{EnvCheckError, Result};
pub use output::{Format, OutputFormatter};
pub use parser::{EnvFile, EnvVar, K8sEnvRef, K8sManifest};
pub use rules::{Diagnostic, Rule, RuleId, Severity};
