use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::error::{EnvCheckError, Result};

/// Represents a source of an environment variable in K8s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum K8sRefSource {
    /// Defined in `env:` list directly.
    Direct,
    /// Sourced from a Secret (`secretKeyRef`).
    SecretKeyRef { name: String, key: String },
    /// Sourced from a ConfigMap (`configMapKeyRef`).
    ConfigMapKeyRef { name: String, key: String },
    /// Defined in a ConfigMap `data` block.
    ConfigMapData,
    /// Defined in a Secret `stringData` or `data` block.
    SecretData,
    /// Reference to a generic envFrom source (Secret or ConfigMap).
    EnvFrom { name: String, kind: String },
}

/// A reference to an environment variable in a K8s manifest.
#[derive(Debug, Clone)]
pub struct K8sEnvRef {
    pub key: String,
    pub source: K8sRefSource,
}

/// Represents a parsed Kubernetes manifest.
#[derive(Debug, Clone)]
pub struct K8sManifest {
    pub path: PathBuf,
    pub kind: String,
    pub name: String,
    /// Environment variables found in this manifest.
    pub env_refs: Vec<K8sEnvRef>,
}

impl K8sManifest {
    pub fn parse(path: impl Into<PathBuf>) -> Result<Vec<Self>> {
        let path = path.into();
        let content = fs::read_to_string(&path).map_err(|e| EnvCheckError::read_error(&path, e))?;

        // YAML file can contain multiple documents separated by "---"
        let mut manifests = Vec::new();

        for doc_str in content.split("\n---") {
            if doc_str.trim().is_empty() {
                continue;
            }

            match serde_yaml::from_str::<Value>(doc_str) {
                Ok(doc) => {
                    if let Some(manifest) = Self::parse_doc(&path, doc) {
                        manifests.push(manifest);
                    }
                },
                Err(e) => {
                    // Only error if the document looks like it should be valid YAML
                    // but fails. For now, strict failure.
                    return Err(EnvCheckError::yaml_error(&path, e));
                },
            }
        }

        Ok(manifests)
    }

    fn parse_doc(path: &Path, doc: Value) -> Option<Self> {
        let kind = doc.get("kind")?.as_str()?.to_string();
        let metadata = doc.get("metadata")?;
        let name = metadata.get("name")?.as_str()?.to_string();

        let mut env_refs = Vec::new();

        match kind.as_str() {
            "Deployment" | "StatefulSet" | "DaemonSet" | "Job" | "CronJob" | "Pod" => {
                extract_pod_spec_env(&doc, &mut env_refs);
            },
            "ConfigMap" => {
                extract_config_map_data(&doc, &mut env_refs);
            },
            "Secret" => {
                extract_secret_data(&doc, &mut env_refs);
            },
            _ => {},
        }

        Some(Self {
            path: path.to_path_buf(),
            kind,
            name,
            env_refs,
        })
    }
}

fn extract_pod_spec_env(doc: &Value, refs: &mut Vec<K8sEnvRef>) {
    // Navigate strictly to spec.template.spec.containers for workloads
    // Or just spec.containers for Pods.
    // Simplifying recursion or paths for now using a recursive search might be overkill.
    // Let's look for standard paths.

    let containers = if let Some(spec) = doc.get("spec") {
        if let Some(template) = spec.get("template") {
            if let Some(pod_spec) = template.get("spec") {
                pod_spec.get("containers")
            } else {
                None
            }
        } else {
            spec.get("containers") // Direct Pod spec
        }
    } else {
        None
    };

    if let Some(Value::Sequence(containers)) = containers {
        for container in containers {
            // Process `env:`
            if let Some(Value::Sequence(env_list)) = container.get("env") {
                for item in env_list {
                    if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                        #[allow(clippy::option_if_let_else)]
                        let source = if let Some(val_from) = item.get("valueFrom") {
                            if let Some(secret_ref) = val_from.get("secretKeyRef") {
                                let s_name = secret_ref
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                let s_key = secret_ref
                                    .get("key")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                K8sRefSource::SecretKeyRef {
                                    name: s_name.into(),
                                    key: s_key.into(),
                                }
                            } else if let Some(cm_ref) = val_from.get("configMapKeyRef") {
                                let cm_name = cm_ref
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                let cm_key = cm_ref
                                    .get("key")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                K8sRefSource::ConfigMapKeyRef {
                                    name: cm_name.into(),
                                    key: cm_key.into(),
                                }
                            } else {
                                K8sRefSource::Direct
                            }
                        } else {
                            K8sRefSource::Direct
                        };

                        refs.push(K8sEnvRef {
                            key: name.to_string(),
                            source,
                        });
                    }
                }
            }

            // Process `envFrom:`
            if let Some(Value::Sequence(env_from_list)) = container.get("envFrom") {
                for item in env_from_list {
                    if let Some(secret_ref) = item.get("secretRef") {
                        if let Some(name) = secret_ref.get("name").and_then(|v| v.as_str()) {
                            refs.push(K8sEnvRef {
                                key: "SECRET_REF:*".to_string(), // Wildcard or specific marker
                                source: K8sRefSource::EnvFrom {
                                    name: name.into(),
                                    kind: "Secret".into(),
                                },
                            });
                        }
                    }
                    if let Some(cm_ref) = item.get("configMapRef") {
                        if let Some(name) = cm_ref.get("name").and_then(|v| v.as_str()) {
                            refs.push(K8sEnvRef {
                                key: "CM_REF:*".to_string(),
                                source: K8sRefSource::EnvFrom {
                                    name: name.into(),
                                    kind: "ConfigMap".into(),
                                },
                            });
                        }
                    }
                }
            }
        }
    }
}

fn extract_config_map_data(doc: &Value, refs: &mut Vec<K8sEnvRef>) {
    if let Some(Value::Mapping(data)) = doc.get("data") {
        for (k, _) in data {
            if let Some(key) = k.as_str() {
                refs.push(K8sEnvRef {
                    key: key.to_string(),
                    source: K8sRefSource::ConfigMapData,
                });
            }
        }
    }
}

fn extract_secret_data(doc: &Value, refs: &mut Vec<K8sEnvRef>) {
    // Check both stringData and data
    for field in ["stringData", "data"] {
        if let Some(Value::Mapping(data)) = doc.get(field) {
            for (k, _) in data {
                if let Some(key) = k.as_str() {
                    refs.push(K8sEnvRef {
                        key: key.to_string(),
                        source: K8sRefSource::SecretData,
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
    fn test_parse_deployment() {
        let yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: DB_HOST
          value: localhost
        - name: DB_PASS
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: password
"#;
        let val: Value = serde_yaml::from_str(yaml).unwrap();
        let manifest = K8sManifest::parse_doc(Path::new("test.yaml"), val).unwrap();

        assert_eq!(manifest.env_refs.len(), 2);
        assert_eq!(manifest.env_refs[0].key, "DB_HOST");
        assert_eq!(manifest.env_refs[1].key, "DB_PASS");
        match &manifest.env_refs[1].source {
            K8sRefSource::SecretKeyRef { name, key } => {
                assert_eq!(name, "db-secret");
                assert_eq!(key, "password");
            },
            _ => panic!("Wrong source"),
        }
    }
}
