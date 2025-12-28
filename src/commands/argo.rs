use crate::error::Result;
use crate::output::Format;
use crate::parser::argocd::EnvSource;
use crate::parser::{argocd, env};
use colored::*;
use std::collections::HashSet;
use std::path::Path;

pub fn run(dir: &Path, env_path: &Path, _format: Format) -> Result<()> {
    // 1. Parse Argo Application manifests
    let refs = argocd::parse_directory(dir)?;

    // 2. Parse .env file
    let env_file = env::EnvFile::parse(env_path)?;
    let env_entries = env_file.vars;

    let env_keys: HashSet<String> = env_entries.into_iter().map(|e| e.key).collect();

    // 3. Compare
    let mut missing_in_env = Vec::new();

    for reference in &refs {
        if !env_keys.contains(&reference.env_var) {
            missing_in_env.push(reference);
        }
    }

    // Report
    if missing_in_env.is_empty() {
        println!("{}", "All ArgoCD env vars found in .env!".green());
        return Ok(());
    }

    println!("{}", "Missing ArgoCD env vars in .env:".red().bold());
    for reference in missing_in_env {
        let source_label = match reference.source {
            EnvSource::Plugin => "plugin",
            EnvSource::Kustomize => "kustomize",
        };
        println!(
            "  {} {} ({} in {})",
            "-".red(),
            reference.env_var.bold(),
            source_label.cyan(),
            reference.path.display()
        );
    }

    Ok(())
}
