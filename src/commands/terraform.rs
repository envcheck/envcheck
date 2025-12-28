use crate::error::Result;
use crate::output::Format;
use crate::parser::{env, terraform};
use colored::*;
use std::path::Path;

pub fn run(dir: &Path, env_path: &Path, _format: Format) -> Result<()> {
    // 1. Parse Terraform directory
    let tf_vars = terraform::parse_directory(dir)?;

    // 2. Parse .env file
    let env_file = env::EnvFile::parse(env_path)?;
    let env_entries = env_file.vars;

    // 3. Compare
    // Terraform `variable "foo"` expects `TF_VAR_foo` in environment (if generic)
    // or just "foo" if using .tfvars (but we are checking .env compatibility)
    // Convention: If users use .env to control Terraform, they typically use `TF_VAR_name`.

    let mut missing_in_env = Vec::new();

    for tf_var in &tf_vars {
        let expected_env_key = format!("TF_VAR_{}", tf_var.name);
        // Check if ANY key in .env matches this
        let mut found = false;
        for entry in &env_entries {
            if entry.key == expected_env_key {
                found = true;
                break;
            }
        }

        if !found {
            missing_in_env.push((tf_var, expected_env_key));
        }
    }

    // Report
    if missing_in_env.is_empty() {
        println!("{}", "All Terraform variables found in .env!".green());
        return Ok(());
    }

    println!("{}", "Missing Terraform variables in .env:".red().bold());
    for (tf_var, expected_key) in missing_in_env {
        println!(
            "  {} {} (defined in {})",
            "-".red(),
            expected_key.bold(),
            tf_var.path.display()
        );
    }

    // TODO: Exit code? using `envcheck` standard logic.
    // If command logic returns Result<()>, main.rs handles success.
    // We should maybe return an error if issues found?
    // Current `lint` returns Ok(()) but prints issues, then main checks error counts?
    // `k8s-sync` returns Ok(()) but prints.
    // Ideally we should return a dedicated error like `LintFailed` if we want non-zero exit code.
    // But `main.rs` doesn't see our internal counts.
    // Let's rely on printing for now or return a generic error if strict.

    // For consistency with other commands, we might want to return `Ok(())`
    // and let the user visually check, OR strictly fail.
    // `lint` uses a Result return for critical failures, but warnings are just printed?
    // Actually `lint` command returns `Result<()>`.

    Ok(())
}
