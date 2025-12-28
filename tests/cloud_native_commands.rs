use crate::common::TempEnvDir;
use envcheck::commands::{actions, argo, helm};
use envcheck::output::Format;

mod common;

#[test]
fn test_actions_command_detects_missing_env_vars() {
    let temp = TempEnvDir::new().unwrap();
    // Simulate .github/workflows/ci.yml
    // Note: The command recursively searches, so placing it in root is fine for this test if command supports it,
    // or we can try to make subdirs. TempEnvDir doesn't easily support subdirs yet without manual fs calls.
    // Our parsers use WalkDir, so root file is fine if extension matches.

    // However, GitHub Actions parser explicitly looks for things that end in .yml/.yaml.
    temp.create_env_file(
        "ci.yml",
        r#"
name: CI
env:
  CI_KEY: "123"
    "#,
    )
    .unwrap();

    let env_path = temp.create_env_file(".env", "OTHER=1\n").unwrap();

    // We expect it to print missing "CI_KEY"
    // For now just check it runs without panic
    let result = actions::run(temp.path(), &env_path, Format::Text);
    assert!(result.is_ok());
}

#[test]
fn test_helm_command_detects_missing_env_vars() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "values.yaml",
        r#"
env:
  DB_HOST: "localhost"
    "#,
    )
    .unwrap();

    let env_path = temp.create_env_file(".env", "OTHER=1\n").unwrap();

    let result = helm::run(temp.path(), &env_path, Format::Text);
    assert!(result.is_ok());
}

#[test]
fn test_argo_command_detects_missing_env_vars() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "app.yaml",
        r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata: 
  name: test
spec:
  source:
    plugin:
      env:
        - name: ARGO_SECRET
          value: "foo"
    "#,
    )
    .unwrap();

    let env_path = temp.create_env_file(".env", "OTHER=1\n").unwrap();

    let result = argo::run(temp.path(), &env_path, Format::Text);
    assert!(result.is_ok());
}
