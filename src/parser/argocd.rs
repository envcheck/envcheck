use crate::error::{EnvCheckError, Result};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct ArgoCDEnvRef {
    pub env_var: String,
    pub path: PathBuf,
}

pub fn parse_directory(dir: &Path) -> Result<Vec<ArgoCDEnvRef>> {
    let mut refs = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext == "yml" || ext == "yaml")
        {
            let content =
                fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;
            if let Ok(value) = serde_yaml::from_str::<Value>(&content) {
                // Check if kind: Application
                if is_argocd_app(&value) {
                    find_plugin_envs(&value, path, &mut refs);
                }
            }
        }
    }

    Ok(refs)
}

fn is_argocd_app(value: &Value) -> bool {
    // kind: Application AND apiVersion: argoproj.io/*
    let kind = value.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let api = value
        .get("apiVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    kind == "Application" && api.starts_with("argoproj.io")
}

fn find_plugin_envs(value: &Value, path: &Path, refs: &mut Vec<ArgoCDEnvRef>) {
    // spec.source.plugin.env name/value
    // traverse to spec -> source -> plugin -> env
    if let Some(spec) = value.get("spec") {
        if let Some(source) = spec.get("source") {
            if let Some(plugin) = source.get("plugin") {
                if let Some(env_list) = plugin.get("env") {
                    if let Some(list) = env_list.as_sequence() {
                        for item in list {
                            if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                                refs.push(ArgoCDEnvRef {
                                    env_var: name.to_string(),
                                    path: path.to_path_buf(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
