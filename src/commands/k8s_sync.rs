use std::collections::HashSet;
use std::io;
use std::path::Path;

use crate::error::{EnvCheckError, Result};
use crate::output::{write_diagnostics, Format};
use crate::parser::{EnvFile, K8sManifest, K8sRefSource};
use crate::rules::{Diagnostic, RuleId, Severity};

pub fn run(manifest_patterns: &[String], env_path: &Path, format: Format) -> Result<()> {
    // 1. Parse .env file
    let env_file = EnvFile::parse(env_path)?;
    let env_keys: HashSet<String> = env_file.vars.iter().map(|v| v.key.clone()).collect();

    // 2. Parse all K8s manifests
    let mut manifests = Vec::new();

    for pattern in manifest_patterns {
        for entry in glob::glob(pattern).map_err(|e| EnvCheckError::GlobError {
            pattern: pattern.clone(),
            source: e,
        })? {
            match entry {
                Ok(path) => {
                    // Try parsing as K8s manifest
                    // If it's a directory, skip? Glob usually returns files.
                    if path.is_file() {
                        let parsed = K8sManifest::parse(path)?;
                        manifests.extend(parsed);
                    }
                },
                Err(e) => return Err(EnvCheckError::read_error("glob entry", io::Error::other(e))),
            }
        }
    }

    if manifests.is_empty() {
        return Err(EnvCheckError::NoFilesMatched {
            pattern: manifest_patterns.join(", "),
        });
    }

    let mut diagnostics = Vec::new();

    // 3. Analyze Mismatches
    // W005: K8s env/secret not in .env
    // W006: .env key not referenced in K8s (Info)

    // Collect all keys used/defined in K8s
    let mut k8s_defined_keys: HashSet<String> = HashSet::new(); // Secrets/ConfigMaps definitions
    let mut k8s_referenced_keys: HashSet<String> = HashSet::new(); // Deployment usage

    for m in &manifests {
        for env_ref in &m.env_refs {
            match &env_ref.source {
                // Definitions
                K8sRefSource::SecretData | K8sRefSource::ConfigMapData => {
                    k8s_defined_keys.insert(env_ref.key.clone());
                },
                // Usages
                K8sRefSource::Direct
                | K8sRefSource::SecretKeyRef { .. }
                | K8sRefSource::ConfigMapKeyRef { .. }
                | K8sRefSource::EnvFrom { .. } => {
                    // Note: Direct env vars (name: value) are sort of definitions+usage locally.
                    // But usually we care if they match .env.
                    // If I have `name: DB_URL, value: ...`, is that a key I expect in .env?
                    // Probably yes, if we want consistency.
                    // But strictly, K8s secrets are the main thing to sync.
                    // The requirement says: "POSTGRES_PASSWORD in k8s ... but not in .env"

                    // If it's a specific key, add to referenced
                    if !env_ref.key.contains('*') {
                        k8s_referenced_keys.insert(env_ref.key.clone());
                    }
                },
            }
        }
    }

    // Check W005: Key defined in K8s (Secret/CM) but missing in .env
    // We scan k8s_defined_keys.
    for k8s_key in &k8s_defined_keys {
        if !env_keys.contains(k8s_key) {
            // Find where it was defined for better error message?
            // We lost the source mapping in the HashSet.
            // Let's iterate manifests again or just report generic errors?
            // Better: Iterate manifests to report precise location.
            for m in &manifests {
                for r in &m.env_refs {
                    if &r.key == k8s_key
                        && matches!(
                            r.source,
                            K8sRefSource::SecretData | K8sRefSource::ConfigMapData
                        )
                    {
                        diagnostics.push(Diagnostic {
                            id: RuleId::W005,
                            severity: Severity::Warning,
                            message: format!(
                                "Key '{}' found in K8s {}/{} but missing in .env",
                                k8s_key, m.kind, m.name
                            ),
                            path: m.path.clone(),
                            line: None, // YAML parser didn't give lines, could add later
                        });
                    }
                }
            }
        }
    }

    // Check W005 for Referenced Keys too?
    // If a deployment uses `CHECK_KEY`, it should probably be in .env?
    // Example: "POSTGRES_PASSWORD in k8s/base/langfuse.yaml but not in .env.example"
    // This implies checking usages too.
    for k8s_key in &k8s_referenced_keys {
        if !env_keys.contains(k8s_key) {
            for m in &manifests {
                for r in &m.env_refs {
                    if &r.key == k8s_key
                        && !matches!(
                            r.source,
                            K8sRefSource::SecretData | K8sRefSource::ConfigMapData
                        )
                    {
                        diagnostics.push(Diagnostic {
                            id: RuleId::W005,
                            severity: Severity::Warning,
                            message: format!(
                                "Key '{}' referenced in K8s {}/{} but missing in .env",
                                k8s_key, m.kind, m.name
                            ),
                            path: m.path.clone(),
                            line: None,
                        });
                    }
                }
            }
        }
    }

    // Check W006: Key in .env not found in K8s (Info)
    // "GROQ_API_KEY in .env.example but not in any K8s manifest"
    let all_k8s_keys: HashSet<_> = k8s_defined_keys.union(&k8s_referenced_keys).collect();

    for env_key in &env_keys {
        if !all_k8s_keys.contains(env_key) {
            // Find line in .env
            let line = env_file
                .vars
                .iter()
                .find(|v| &v.key == env_key)
                .map(|v| v.line);

            diagnostics.push(Diagnostic {
                id: RuleId::W006,
                severity: Severity::Info,
                message: format!("Key '{env_key}' in .env but not found in any K8s manifest"),
                path: env_file.path.clone(),
                line,
            });
        }
    }

    // Deduplicate diagnostics? (Same key in multiple manifests might spam)
    // For now keep all.

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_diagnostics(format, &diagnostics, &mut handle)
        .map_err(|e| EnvCheckError::read_error("stdout", e))?;

    Ok(())
}
