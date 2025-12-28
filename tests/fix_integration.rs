#![allow(deprecated)]
use assert_cmd::Command;
use std::fs;

mod common;

/// Helper to get the envcheck binary command
fn envcheck_cmd() -> Command {
    Command::cargo_bin("envcheck").expect("Failed to find envcheck binary")
}

#[test]
fn test_fix_sorts_keys() {
    let temp = common::TempEnvFile::new("B_KEY=2\nA_KEY=1\n").unwrap();
    let path = temp.path();

    envcheck_cmd().arg("fix").arg(path).assert().success();

    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, "A_KEY=1\nB_KEY=2\n");
}

#[test]
fn test_fix_trims_whitespace() {
    let temp = common::TempEnvFile::new("KEY=value  \n").unwrap();
    let path = temp.path();

    envcheck_cmd().arg("fix").arg(path).assert().success();

    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, "KEY=value\n");
}

#[test]
fn test_fix_preserves_comments() {
    let input = "# Header\n\n# Key B\nB_KEY=2\n\n# Key A\nA_KEY=1\n# Footer";
    let temp = common::TempEnvFile::new(input).unwrap();
    let path = temp.path();

    envcheck_cmd().arg("fix").arg(path).assert().success();

    let content = fs::read_to_string(path).unwrap();

    // Exact formatting depends on implementation (newlines between entries?)
    // Our implementation puts one newline after each entry block
    // So:
    // Entry Header: "# Header\n"
    // Entry A: "# Key A\nA_KEY=1\n"
    // Entry B: "# Key B\nB_KEY=2\n"
    // Footer: "# Footer\n"

    // Wait, parser groups comments.
    // If blank lines existed, parser kept them?
    // My parser implementation:
    // `current_comments.push(trimmed)` -> trimmed blank line is empty string?
    // No, `if trimmed.is_empty() { continue }`
    // So blank lines are LOST.

    // Thus comments will be packed tight.
    // Output:
    // # Header
    // # Key A
    // A_KEY=1
    // # Key B
    // B_KEY=2
    // # Footer

    // Let's verify THAT structure.
    let expected_tight = "# Header\n# Key A\nA_KEY=1\n# Key B\nB_KEY=2\n# Footer\n";
    assert_eq!(content, expected_tight);
}
