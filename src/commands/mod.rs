pub mod compare;
pub mod doctor;
pub mod fix;
pub mod k8s_sync;
pub mod lint;

use clap::Subcommand;
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

    /// Lint all .env files in the current directory
    Doctor,
}
