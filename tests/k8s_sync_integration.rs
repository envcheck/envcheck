#![allow(deprecated)]
//! Integration tests for the `envcheck k8s-sync` command
//!
//! Tests detecting mismatches between K8s manifests and .env files.

mod common;
use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get the envcheck binary command
fn envcheck_cmd() -> Command {
    Command::cargo_bin("envcheck").expect("Failed to find envcheck binary")
}

#[test]
fn test_k8s_sync_detects_secret_not_in_env() {
    let deployment = common::fixture_path("k8s", "deployment.yaml");
    let env_file = common::fixture_path("env", "example.env");

    // deployment.yaml has POSTGRES_PASSWORD which is not in example.env
    envcheck_cmd()
        .arg("k8s-sync")
        .arg(&deployment)
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success() // Warning W005
        .stdout(predicate::str::contains("W005"))
        .stdout(predicate::str::contains("POSTGRES_PASSWORD"));
}

#[test]
fn test_k8s_sync_detects_env_not_in_k8s() {
    let configmap = common::fixture_path("k8s", "configmap.yaml");
    let env_file = common::fixture_path("env", "example.env");

    // example.env has GROQ_API_KEY which is not in configmap.yaml
    envcheck_cmd()
        .arg("k8s-sync")
        .arg(&configmap)
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success() // Info W006
        .stdout(predicate::str::contains("W006").or(predicate::str::contains("GROQ_API_KEY")));
}

#[test]
fn test_k8s_sync_multiple_manifests() {
    let deployment = common::fixture_path("k8s", "deployment.yaml");
    let configmap = common::fixture_path("k8s", "configmap.yaml");
    let secret = common::fixture_path("k8s", "secret.yaml");
    let env_file = common::fixture_path("env", "example.env");

    envcheck_cmd()
        .arg("k8s-sync")
        .args([&deployment, &configmap, &secret])
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success(); // Should detect mismatches but output warnings
}

#[test]
fn test_k8s_sync_with_temp_manifest() {
    let manifest_content = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config
data:
  UNIQUE_K8S_KEY: "value"
"#;

    let temp_manifest =
        common::TempK8sManifest::new(manifest_content).expect("Failed to create temp manifest");

    let env_file = common::fixture_path("env", "valid.env");

    // UNIQUE_K8S_KEY is not in valid.env
    envcheck_cmd()
        .arg("k8s-sync")
        .arg(temp_manifest.path())
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("UNIQUE_K8S_KEY"));
}

#[test]
fn test_k8s_sync_extracts_secret_key_ref() {
    let secret = common::fixture_path("k8s", "secret.yaml");
    let env_file = common::fixture_path("env", "valid.env");

    // secret.yaml has JWT_SECRET which is not in valid.env
    envcheck_cmd()
        .arg("k8s-sync")
        .arg(&secret)
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("JWT_SECRET"));
}

#[test]
fn test_k8s_sync_json_output() {
    let deployment = common::fixture_path("k8s", "deployment.yaml");
    let env_file = common::fixture_path("env", "example.env");

    envcheck_cmd()
        .arg("k8s-sync")
        .arg(&deployment)
        .arg("--env")
        .arg(&env_file)
        .arg("--format=json")
        .assert()
        .success()
        .stdout(predicate::str::contains("W005")); // Check for rule ID
}

#[test]
fn test_k8s_sync_glob_pattern() {
    let fixtures_dir = common::fixtures_dir();
    let k8s_glob = fixtures_dir.join("k8s").join("*.yaml");
    let env_file = common::fixture_path("env", "example.env");

    envcheck_cmd()
        .arg("k8s-sync")
        .arg(k8s_glob.to_string_lossy().to_string())
        .arg("--env")
        .arg(&env_file)
        .assert()
        .success();
}

#[test]
fn test_k8s_sync_requires_env_flag() {
    let deployment = common::fixture_path("k8s", "deployment.yaml");

    envcheck_cmd()
        .arg("k8s-sync")
        .arg(&deployment)
        // Missing --env flag
        .assert()
        .failure()
        .stderr(predicate::str::contains("--env"));
}
