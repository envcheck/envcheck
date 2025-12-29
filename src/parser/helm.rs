use crate::error::{EnvCheckError, Result};
use regex::Regex;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelmEnvRef {
    pub env_var: String,
    pub path: PathBuf,
}

pub fn parse_directory(dir: &Path) -> Result<Vec<HelmEnvRef>> {
    let mut refs = Vec::new();
    let re = Regex::new(r"^[A-Z][A-Z0-9_]+$")
        .map_err(|e| EnvCheckError::parse_error(PathBuf::from("regex"), 0, e.to_string()))?;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        // Look for values.yaml or *-values.yaml
        let fname = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        if path.is_file() && (fname == "values.yaml" || fname.ends_with("-values.yaml")) {
            let content =
                fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;
            if let Ok(value) = serde_yaml::from_str::<Value>(&content) {
                find_uppercase_keys(&value, path, &mut refs, &re);
            }
        }
    }

    Ok(refs)
}

fn find_uppercase_keys(value: &Value, path: &Path, refs: &mut Vec<HelmEnvRef>, re: &Regex) {
    match value {
        Value::Mapping(map) => {
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    // Check if key is SCREAMING_SNAKE_CASE
                    if re.is_match(key_str) {
                        refs.push(HelmEnvRef {
                            env_var: key_str.to_string(),
                            path: path.to_path_buf(),
                        });
                    }
                }
                find_uppercase_keys(v, path, refs, re);
            }
        },
        Value::Sequence(seq) => {
            for v in seq {
                find_uppercase_keys(v, path, refs, re);
            }
        },
        _ => {},
    }
}
