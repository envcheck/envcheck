pub mod actions;
pub mod ansible;
pub mod argo;
pub mod compare;
pub mod completions;
pub mod doctor;
pub mod fix;
pub mod helm;
pub mod k8s_sync;
pub mod lint;
pub mod terraform;

use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Lint .env files
    Lint {
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Fix .env files (sort keys, remove whitespace)
    Fix {
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Auto-commit fixes to current branch
        #[arg(long)]
        commit: bool,

        /// Create a PR with fixes (requires gh CLI)
        #[arg(long)]
        pr: bool,
    },
    /// Compare .env files
    Compare {
        /// Reference file (e.g. .env.example)
        #[arg(required = true, num_args = 2..)]
        files: Vec<PathBuf>,
    },

    /// Detect mismatches between K8s manifests and .env files
    #[command(name = "k8s-sync")]
    K8sSync {
        /// K8s manifest files or glob patterns
        #[arg(required = true)]
        manifests: Vec<String>,

        /// The .env file to check against
        #[arg(long)]
        env: PathBuf,
    },

    /// Check Terraform variables
    Terraform(TerraformArgs),

    /// Check Ansible playbooks
    Ansible(AnsibleArgs),

    /// Check GitHub Actions workflows
    Actions(ActionsArgs),

    /// Check Helm Chart values
    Helm(HelmArgs),

    /// Check ArgoCD Application manifests
    Argo(ArgoArgs),

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Lint all .env files in the current directory
    Doctor,
}

#[derive(Args, Debug)]
pub struct TerraformArgs {
    /// Directory to scan for .tf files
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Path to .env file
    #[arg(long, short, default_value = ".env")]
    pub env: PathBuf,
}

#[derive(Args, Debug)]
pub struct AnsibleArgs {
    /// Directory to scan for Ansible files
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Path to .env file
    #[arg(long, short, default_value = ".env")]
    pub env: PathBuf,
}

#[derive(Args, Debug)]
pub struct ActionsArgs {
    /// Directory to scan for workflow files (defaults to .github/workflows if exists, else .)
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Path to .env file
    #[arg(long, short, default_value = ".env")]
    pub env: PathBuf,
}

#[derive(Args, Debug)]
pub struct HelmArgs {
    /// Directory to scan for values.yaml files
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Path to .env file
    #[arg(long, short, default_value = ".env")]
    pub env: PathBuf,
}

#[derive(Args, Debug)]
pub struct ArgoArgs {
    /// Directory to scan for Application manifests
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Path to .env file
    #[arg(long, short, default_value = ".env")]
    pub env: PathBuf,
}
