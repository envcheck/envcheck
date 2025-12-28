use crate::common::TempEnvDir;
use envcheck::parser::github_actions;

mod common;

#[test]
fn test_actions_parser_finds_env_vars() {
    let temp = TempEnvDir::new().unwrap();
    // Create a mock workflow file
    // Note: We need to recreate the .github/workflows structure if we enforce it,
    // but our parser currently scans all YAMLs in the target dir.
    temp.create_env_file(
        "ci.yml",
        r#"
name: CI
on: [push]
env:
  GLOBAL_VAR: "value"

jobs:
  build:
    runs-on: ubuntu-latest
    env:
      JOB_VAR: "job_value"
    steps:
      - name: Run checks
        env:
          STEP_VAR: "step_value"
        run: echo "Hello"
    "#,
    )
    .unwrap();

    let path = temp.path();
    let vars = github_actions::parse_directory(path).unwrap();

    assert_eq!(vars.len(), 3);
    let names: Vec<String> = vars.iter().map(|v| v.env_var.clone()).collect();
    assert!(names.contains(&"GLOBAL_VAR".to_string()));
    assert!(names.contains(&"JOB_VAR".to_string()));
    assert!(names.contains(&"STEP_VAR".to_string()));
}
