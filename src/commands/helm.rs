use crate::error::Result;
use crate::output::Format;
use crate::parser::{env, helm};
use colored::*;
use std::collections::HashSet;
use std::path::Path;

pub fn run(dir: &Path, env_path: &Path, _format: Format) -> Result<()> {
    // 1. Parse Helm values
    let refs = helm::parse_directory(dir)?;

    // 2. Parse .env file
    let env_file = env::EnvFile::parse(env_path)?;
    let env_entries = env_file.vars;

    let env_keys: HashSet<String> = env_entries.into_iter().map(|e| e.key).collect();

    // 3. Compare
    let mut missing_in_env = Vec::new();

    for reference in &refs {
        // We look for direct match: KEY in values.yaml should mean KEY in .env
        if !env_keys.contains(&reference.env_var) {
            missing_in_env.push(reference);
        }
    }

    // Report
    if missing_in_env.is_empty() {
        println!("{}", "All Helm values.yaml env vars found in .env!".green());
        return Ok(());
    }

    println!(
        "{}",
        "Missing Helm values.yaml env vars in .env:".red().bold()
    );
    for reference in missing_in_env {
        println!(
            "  {} {} (defined in {})",
            "-".red(),
            reference.env_var.bold(),
            reference.path.display()
        );
    }

    Ok(())
}
