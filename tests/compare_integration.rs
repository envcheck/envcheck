#![allow(deprecated)]
//! Integration tests for the `envcheck compare` command
//!
//! Tests comparing keys across multiple .env files.

mod common;
use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get the envcheck binary command
fn envcheck_cmd() -> Command {
    Command::cargo_bin("envcheck").expect("Failed to find envcheck binary")
}

#[test]
fn test_compare_identical_files() {
    let fixture = common::fixture_path("env", "valid.env");

    envcheck_cmd()
        .arg("compare")
        .args([&fixture, &fixture])
        .assert()
        .success();
}

#[test]
fn test_compare_detects_missing_keys() {
    // Create temporary files with different keys
    let temp_dir = common::TempEnvDir::new().expect("Failed to create temp dir");

    let example = temp_dir
        .create_env_file(".env.example", "KEY_A=value\nKEY_B=value\nKEY_C=value\n")
        .unwrap();

    let local = temp_dir
        .create_env_file(
            ".env.local",
            "KEY_A=value\nKEY_B=value\n", // Missing KEY_C
        )
        .unwrap();

    envcheck_cmd()
        .arg("compare")
        .args([&example, &local])
        .assert()
        .success() // W004 is a warning, so exit code 0
        .stdout(predicate::str::contains("KEY_C"))
        .stdout(predicate::str::contains("W004").or(predicate::str::contains("missing")));
}

#[test]
fn test_compare_multiple_files() {
    let temp_dir = common::TempEnvDir::new().expect("Failed to create temp dir");

    let example = temp_dir
        .create_env_file(".env.example", "DB_HOST=\nDB_PORT=\nAPI_KEY=\n")
        .unwrap();

    let local = temp_dir
        .create_env_file(
            ".env.local",
            "DB_HOST=localhost\nDB_PORT=5432\n", // Missing API_KEY
        )
        .unwrap();

    let prod = temp_dir
        .create_env_file(
            ".env.prod",
            "DB_HOST=prod-db\nAPI_KEY=secret\n", // Missing DB_PORT
        )
        .unwrap();

    envcheck_cmd()
        .arg("compare")
        .args([&example, &local, &prod])
        .assert()
        .success()
        .stdout(predicate::str::contains("API_KEY"))
        .stdout(predicate::str::contains("DB_PORT"));
}

#[test]
fn test_compare_with_fixture_files() {
    let example = common::fixture_path("env", "example.env");
    let valid = common::fixture_path("env", "valid.env");

    // example.env has GROQ_API_KEY which valid.env doesn't have
    envcheck_cmd()
        .arg("compare")
        .args([&example, &valid])
        .assert()
        .success()
        .stdout(predicate::str::contains("GROQ_API_KEY"));
}

#[test]
fn test_compare_json_output() {
    let temp_dir = common::TempEnvDir::new().expect("Failed to create temp dir");

    let file1 = temp_dir.create_env_file("one.env", "KEY_A=1\n").unwrap();
    let file2 = temp_dir.create_env_file("two.env", "KEY_B=2\n").unwrap();

    envcheck_cmd()
        .arg("compare")
        .arg("--format=json")
        .args([&file1, &file2])
        .assert()
        .success()
        .stdout(predicate::str::contains("W004")); // Check for rule ID
}

#[test]
fn test_compare_requires_multiple_files() {
    let fixture = common::fixture_path("env", "valid.env");

    envcheck_cmd()
        .arg("compare")
        .arg(&fixture)
        .assert()
        .failure()
        // Clap error message: "error: 2 values required..."
        .stderr(predicate::str::contains("required").and(predicate::str::contains("2")));
}
