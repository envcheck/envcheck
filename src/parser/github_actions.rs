use crate::error::{EnvCheckError, Result};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct GitHubActionEnvRef {
    pub env_var: String,
    pub path: PathBuf,
}

pub fn parse_directory(dir: &Path) -> Result<Vec<GitHubActionEnvRef>> {
    let mut refs = Vec::new();
    // Convention: .github/workflows/*.yml or *.yaml
    // Convention: .github/workflows/*.yml or *.yaml

    // If .github/workflows doesn't exist, maybe the user passed the workflows dir directly?
    // Let's search recursively in the given dir, but filter for .yml/.yaml
    // If the dir is the project root, we should strictly look in .github/workflows to avoid false positives?
    // But `envcheck actions <dir>` might point to a specific dir.
    // Let's just walk the given dir.

    if !dir.exists() {
        return Ok(refs);
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext == "yml" || ext == "yaml")
        {
            // Optimization: Skip loop if not in .github/workflows if we are scanning root?
            // For now, scan all YAMLs? No, that might be too broad.
            // Let's assume the user points to the project root, so we expect .github/workflows.
            // Or they point to the workflows dir.
            // Let's stick to "is it a yaml file".

            // Actually, simpler: Use `serde_yaml` to parse generic Value.
            let content =
                fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;

            if let Ok(value) = serde_yaml::from_str::<Value>(&content) {
                // Recursively find "env" keys
                find_env_vars(&value, path, &mut refs);
            }
        }
    }

    Ok(refs)
}

fn find_env_vars(value: &Value, path: &Path, refs: &mut Vec<GitHubActionEnvRef>) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    if key_str == "env" {
                        // Found an env block. Iterate attributes.
                        if let Value::Mapping(env_map) = v {
                            for (env_key, _) in env_map {
                                if let Some(env_name) = env_key.as_str() {
                                    refs.push(GitHubActionEnvRef {
                                        env_var: env_name.to_string(),
                                        path: path.to_path_buf(),
                                    });
                                }
                            }
                        }
                    }
                }
                // Recurse
                find_env_vars(v, path, refs);
            }
        },
        Value::Sequence(seq) => {
            for v in seq {
                find_env_vars(v, path, refs);
            }
        },
        _ => {},
    }
}
