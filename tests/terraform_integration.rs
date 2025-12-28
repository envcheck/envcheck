use crate::common::TempEnvDir;
use envcheck::parser::terraform;

mod common;

// Note: We need to expose parser logic or test via command?
// Current implementation is internal to `parser/terraform.rs`.
// We should expose it or make it `pub` in `lib.rs`.
// `src/lib.rs` exports parser module. `src/parser/mod.rs` likely does too.
// Let's check `src/parser/mod.rs` visibility.

// Since we are writing integration tests in `tests/`, we can only access pub API of the crate.
// We haven't updated `src/parser/mod.rs` yet.

#[test]
fn test_terraform_command_exists() {
    // Placeholder until we implement the command
}

#[test]
fn test_terraform_parser_finds_variables() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "main.tf",
        r#"
    variable "region" {
      description = "AWS region"
      default      = "us-east-1"
    }
    "#,
    )
    .unwrap();

    temp.create_env_file(
        "variables.tf",
        r#"
    variable "db_password" {
        description = "Database password"
        sensitive   = true
    }
    "#,
    )
    .unwrap();
    let path = temp.path();

    let vars = terraform::parse_directory(path).unwrap();

    // Should find 2 variables
    assert_eq!(vars.len(), 2);

    let names: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
    assert!(names.contains(&"region".to_string()));
    assert!(names.contains(&"db_password".to_string()));
}
