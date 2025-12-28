#![allow(dead_code)]
//! Shared test utilities for envcheck integration tests
//!
//! Provides helper functions for loading fixtures and creating temporary test files.

use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

/// Returns the absolute path to a fixture file.
///
/// # Arguments
/// * `category` - The fixture category ("env" or "k8s")
/// * `name` - The fixture file name (e.g., "valid.env" or "deployment.yaml")
///
/// # Example
/// ```ignore
/// let path = fixture_path("env", "valid.env");
/// assert!(path.exists());
/// ```
pub fn fixture_path(category: &str, name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    Path::new(&manifest_dir)
        .join("tests")
        .join("fixtures")
        .join(category)
        .join(name)
}

/// Returns the path to the fixtures directory.
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    Path::new(&manifest_dir).join("tests").join("fixtures")
}

/// Helper for creating temporary .env files with custom content.
///
/// The file is automatically deleted when the struct is dropped.
pub struct TempEnvFile {
    file: NamedTempFile,
}

impl TempEnvFile {
    /// Creates a new temporary .env file with the given content.
    pub fn new(content: &str) -> std::io::Result<Self> {
        let mut file = NamedTempFile::with_suffix(".env")?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(Self { file })
    }

    /// Returns the path to the temporary file.
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

/// Helper for creating a temporary directory with multiple .env files.
///
/// Useful for testing the `compare` command across multiple files.
pub struct TempEnvDir {
    dir: TempDir,
}

impl TempEnvDir {
    /// Creates a new temporary directory.
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            dir: TempDir::new()?,
        })
    }

    /// Creates a .env file in the temporary directory with the given content.
    pub fn create_env_file(&self, name: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = self.dir.path().join(name);
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Returns the path to the temporary directory.
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
}

/// Helper for creating temporary K8s manifest files.
pub struct TempK8sManifest {
    file: NamedTempFile,
}

impl TempK8sManifest {
    /// Creates a new temporary K8s manifest file with the given YAML content.
    pub fn new(content: &str) -> std::io::Result<Self> {
        let mut file = NamedTempFile::with_suffix(".yaml")?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(Self { file })
    }

    /// Returns the path to the temporary file.
    pub fn path(&self) -> &Path {
        self.file.path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_path_exists() {
        let path = fixture_path("env", "valid.env");
        assert!(path.exists(), "Fixture file should exist: {path:?}");
    }

    #[test]
    fn test_temp_env_file() {
        let content = "TEST_KEY=test_value\n";
        let temp_file = TempEnvFile::new(content).unwrap();

        let read_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_temp_env_dir() {
        let temp_dir = TempEnvDir::new().unwrap();

        let file1 = temp_dir
            .create_env_file(".env.local", "KEY1=value1\n")
            .unwrap();
        let file2 = temp_dir
            .create_env_file(".env.prod", "KEY2=value2\n")
            .unwrap();

        assert!(file1.exists());
        assert!(file2.exists());
    }
}
