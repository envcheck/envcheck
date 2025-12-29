use crate::error::{EnvCheckError, Result};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsibleEnvRef {
    pub env_var: String,
    pub path: PathBuf,
}

pub fn parse_directory(dir: &Path) -> Result<Vec<AnsibleEnvRef>> {
    let mut refs = Vec::new();
    // Regex for: lookup('env', 'VAR') or lookup("env", "VAR")
    // Captures the variable name.
    // Allow whitespace.
    let re = Regex::new(r#"lookup\(\s*['"]env['"]\s*,\s*['"]([^'"]+)['"]\s*\)"#)
        .map_err(|e| EnvCheckError::parse_error(PathBuf::from("regex"), 0, e.to_string()))?;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        // Check for .yml or .yaml
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            let content =
                fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;

            // Search validation for matches
            for cap in re.captures_iter(&content) {
                if let Some(matched) = cap.get(1) {
                    refs.push(AnsibleEnvRef {
                        env_var: matched.as_str().to_string(),
                        path: path.to_path_buf(),
                    });
                }
            }
        }
    }

    Ok(refs)
}
