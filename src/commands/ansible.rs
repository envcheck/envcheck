use crate::error::Result;
use crate::output::Format;
use crate::parser::{ansible, env};
use colored::*;
use std::collections::HashSet;
use std::path::Path;

pub fn run(dir: &Path, env_path: &Path, _format: Format) -> Result<()> {
    // 1. Parse Ansible directory
    let ansible_refs = ansible::parse_directory(dir)?;

    // 2. Parse .env file
    let env_file = env::EnvFile::parse(env_path)?;
    let env_entries = env_file.vars;

    let env_keys: HashSet<String> = env_entries.into_iter().map(|e| e.key).collect();

    // 3. Compare
    let mut missing_in_env = Vec::new();

    for reference in &ansible_refs {
        if !env_keys.contains(&reference.env_var) {
            missing_in_env.push(reference);
        }
    }

    // Report
    if missing_in_env.is_empty() {
        println!("{}", "All Ansible lookups found in .env!".green());
        return Ok(());
    }

    println!("{}", "Missing Ansible env lookups in .env:".red().bold());
    for reference in missing_in_env {
        println!(
            "  {} {} (used in {})",
            "-".red(),
            reference.env_var.bold(),
            reference.path.display()
        );
    }

    Ok(())
}
