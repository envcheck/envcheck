#![allow(deprecated)]
//! Integration tests for the `envcheck lint` command
//!
//! Tests the basic linting functionality.

mod common;

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get the envcheck binary command
fn envcheck_cmd() -> Command {
    Command::cargo_bin("envcheck").expect("Failed to find envcheck binary")
}

#[test]
fn test_lint_valid_file_succeeds() {
    let fixture = common::fixture_path("env", "valid.env");

    envcheck_cmd().arg("lint").arg(&fixture).assert().success();
}

#[test]
fn test_lint_detects_duplicate_keys() {
    let fixture = common::fixture_path("env", "duplicates.env");

    envcheck_cmd()
        .arg("lint")
        .arg(&fixture)
        .assert()
        .failure()
        .stdout(predicate::str::contains("E001")) // Diagnostics on stdout
        .stdout(predicate::str::contains("Duplicate key 'DATABASE_HOST'"))
        .stdout(predicate::str::contains("Duplicate key 'API_KEY'"));
}

#[test]
fn test_lint_detects_invalid_syntax() {
    let fixture = common::fixture_path("env", "invalid_syntax.env");

    // This should fail with E002 diagnostics
    envcheck_cmd()
        .arg("lint")
        .arg(&fixture)
        .assert()
        .failure()
        .stdout(predicate::str::contains("E002"));
}

#[test]
fn test_lint_warns_empty_values() {
    let fixture = common::fixture_path("env", "empty_values.env");

    // Warnings only -> exit code 0 or 2 depending on logic?
    // EnvCheckError logic: LintFailed with 0 errors but warnings -> 0 exit code.
    // So assert success.
    envcheck_cmd()
        .arg("lint")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("W001"));
}

#[test]
fn test_lint_warns_trailing_whitespace() {
    let fixture = common::fixture_path("env", "trailing_whitespace.env");

    envcheck_cmd()
        .arg("lint")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("W002"));
}

#[test]
fn test_lint_multiple_files() {
    let valid = common::fixture_path("env", "valid.env");
    let duplicates = common::fixture_path("env", "duplicates.env");

    envcheck_cmd()
        .arg("lint")
        .args([&valid, &duplicates])
        .assert()
        .failure()
        .stdout(predicate::str::contains("E001"));
}

#[test]
fn test_lint_json_output() {
    let fixture = common::fixture_path("env", "duplicates.env");

    envcheck_cmd()
        .arg("lint")
        .arg("--format=json")
        .arg(&fixture)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"rule\": \"E001\"")); // Corrected JSON string check
}

#[test]
fn test_lint_nonexistent_file() {
    envcheck_cmd()
        .arg("lint")
        .arg("nonexistent.env")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read file"));
}

#[test]
fn test_lint_with_temp_file() {
    // Test helper integration
    let temp = common::TempEnvFile::new("DUPLICATE_KEY=1\nDUPLICATE_KEY=2\n").unwrap();

    envcheck_cmd()
        .arg("lint")
        .arg(temp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("E001"));
}

#[test]
fn test_lint_warns_unsorted_keys() {
    let temp = common::TempEnvFile::new("B_KEY=2\nA_KEY=1\n").unwrap();

    envcheck_cmd()
        .arg("lint")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("W003"))
        .stdout(predicate::str::contains("Unsorted key 'A_KEY'"));
}
