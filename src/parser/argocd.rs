use crate::error::{EnvCheckError, Result};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq)]
pub struct ArgoCDEnvRef {
    pub env_var: String,
    pub path: PathBuf,
    /// Source of the env var (plugin, kustomize)
    pub source: EnvSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvSource {
    Plugin,
    Kustomize,
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
                    find_env_refs(&value, path, &mut refs);
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

fn find_env_refs(value: &Value, path: &Path, refs: &mut Vec<ArgoCDEnvRef>) {
    if let Some(spec) = value.get("spec") {
        // Check source (single source)
        if let Some(source) = spec.get("source") {
            find_source_envs(source, path, refs);
        }

        // Check sources (multi-source applications)
        if let Some(sources) = spec.get("sources") {
            if let Some(sources_list) = sources.as_sequence() {
                for source in sources_list {
                    find_source_envs(source, path, refs);
                }
            }
        }
    }
}

fn find_source_envs(source: &Value, path: &Path, refs: &mut Vec<ArgoCDEnvRef>) {
    // spec.source.plugin.env[].name
    if let Some(plugin) = source.get("plugin") {
        if let Some(env_list) = plugin.get("env") {
            if let Some(list) = env_list.as_sequence() {
                for item in list {
                    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                        refs.push(ArgoCDEnvRef {
                            env_var: name.to_string(),
                            path: path.to_path_buf(),
                            source: EnvSource::Plugin,
                        });
                    }
                }
            }
        }
    }

    // spec.source.kustomize.commonEnv[].name and spec.source.kustomize.env[].name
    if let Some(kustomize) = source.get("kustomize") {
        // commonEnv - for all resources
        extract_kustomize_env(kustomize.get("commonEnv"), path, refs);

        // env - legacy/alternative field
        extract_kustomize_env(kustomize.get("env"), path, refs);

        // Also check commonEnvs (plural form sometimes used)
        extract_kustomize_env(kustomize.get("commonEnvs"), path, refs);
    }
}

fn extract_kustomize_env(env_value: Option<&Value>, path: &Path, refs: &mut Vec<ArgoCDEnvRef>) {
    if let Some(env_list) = env_value {
        if let Some(list) = env_list.as_sequence() {
            for item in list {
                if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                    refs.push(ArgoCDEnvRef {
                        env_var: name.to_string(),
                        path: path.to_path_buf(),
                        source: EnvSource::Kustomize,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_env() {
        let yaml = r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: test-app
spec:
  source:
    plugin:
      env:
        - name: DATABASE_URL
          value: secret
        - name: API_KEY
          value: test
"#;
        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let mut refs = Vec::new();
        find_env_refs(&value, Path::new("test.yaml"), &mut refs);

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].env_var, "DATABASE_URL");
        assert_eq!(refs[0].source, EnvSource::Plugin);
        assert_eq!(refs[1].env_var, "API_KEY");
    }

    #[test]
    fn test_parse_kustomize_common_env() {
        let yaml = r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: kustomize-app
spec:
  source:
    kustomize:
      commonEnv:
        - name: POSTGRES_PASSWORD
        - name: REDIS_URL
"#;
        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let mut refs = Vec::new();
        find_env_refs(&value, Path::new("test.yaml"), &mut refs);

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].env_var, "POSTGRES_PASSWORD");
        assert_eq!(refs[0].source, EnvSource::Kustomize);
        assert_eq!(refs[1].env_var, "REDIS_URL");
    }

    #[test]
    fn test_parse_multi_source() {
        let yaml = r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: multi-source-app
spec:
  sources:
    - plugin:
        env:
          - name: SOURCE1_VAR
    - kustomize:
        commonEnv:
          - name: SOURCE2_VAR
"#;
        let value: Value = serde_yaml::from_str(yaml).unwrap();
        let mut refs = Vec::new();
        find_env_refs(&value, Path::new("test.yaml"), &mut refs);

        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].env_var, "SOURCE1_VAR");
        assert_eq!(refs[1].env_var, "SOURCE2_VAR");
    }
}
