use crate::error::Result;
use crate::output::Format;
use crate::parser::{env, github_actions};
use colored::*;
use std::collections::HashSet;
use std::path::Path;

pub fn run(dir: &Path, env_path: &Path, _format: Format) -> Result<()> {
    // 1. Parse Directory (recursively scanning for .yml/.yaml)
    // Note: The parser currently scans the whole dir.
    // If the user passes ".", we might want to default to ".github/workflows" if it exists,
    // to avoid false positives from other YAMLs.
    // However, keeping consistent with other commands, we trust the parser/user.
    // Ideally, the command runner handles the path logic.

    // Let's verify if dir/.github/workflows exists, use that if dir is root?
    // Or just pass dir.

    let refs = github_actions::parse_directory(dir)?;

    // 2. Parse .env file
    let env_file = env::EnvFile::parse(env_path)?;
    let env_entries = env_file.vars;

    let env_keys: HashSet<String> = env_entries.into_iter().map(|e| e.key).collect();

    // 3. Compare
    let mut missing_in_env = Vec::new();

    for reference in &refs {
        // GHA `env` keys are typically what we set directly, so they should match .env keys.
        // e.g. `env: MY_KEY: ${{ secrets.MY_KEY }}` -> We want to check if MY_KEY is in .env?
        // Or if the VALUE uses a secret that we should define?
        // Actually, often in local dev we want `MY_KEY=...` in .env.
        // The parser extracts keys from `env:`.

        if !env_keys.contains(&reference.env_var) {
            missing_in_env.push(reference);
        }
    }

    // Report
    if missing_in_env.is_empty() {
        println!("{}", "All GitHub Actions env vars found in .env!".green());
        return Ok(());
    }

    println!(
        "{}",
        "Missing GitHub Actions env vars in .env:".red().bold()
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
