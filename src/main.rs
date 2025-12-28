use std::process::ExitCode;

use clap::Parser;
use envcheck::commands::{self, Commands};
use envcheck::output::Format;

#[derive(Parser, Debug)]
#[command(name = "envcheck")]
#[command(about = "A fast, modern CLI for linting .env files", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format (text, json, github)
    #[arg(long, global = true, default_value = "text")]
    format: Format,

    /// Suppress output
    #[arg(short, long, global = true)]
    quiet: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Initialize logging if needed (e.g. RUST_LOG env var)
    tracing_subscriber::fmt::init();

    let result = match &cli.command {
        Commands::Lint { files } => commands::lint::run(files, cli.format),
        Commands::Compare { files } => commands::compare::run(files, cli.format),
        Commands::Fix { files, commit, pr } => commands::fix::run(files, *commit, *pr),
        Commands::K8sSync { manifests, env } => commands::k8s_sync::run(manifests, env, cli.format),
        Commands::Terraform(args) => commands::terraform::run(&args.dir, &args.env, cli.format),
        Commands::Ansible(args) => commands::ansible::run(&args.dir, &args.env, cli.format),
        Commands::Actions(args) => commands::actions::run(&args.dir, &args.env, cli.format),
        Commands::Helm(args) => commands::helm::run(&args.dir, &args.env, cli.format),
        Commands::Argo(args) => commands::argo::run(&args.dir, &args.env, cli.format),
        Commands::Completions { shell } => commands::completions::run(*shell),
        Commands::Doctor => commands::doctor::run(cli.format),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            // Print error to stderr if not quiet
            if !cli.quiet {
                // Use Display implementation of EnvCheckError
                // If format is JSON, maybe print error as JSON?
                // For now, standard stderr error printing.
                eprintln!("Error: {err}");
            }

            // Return appropriate exit code
            // LintFailed returns 1 (error) or 0 (warning only, dependent on logic)
            // But main.rs acts as the shell interface.
            // If checking exit code from error:
            let code = err.exit_code();
            // Cast i32 to u8 for ExitCode.
            // 1 -> FAILURE?
            // ExitCode doesn't support raw integers easily in stable Rust without specialized crates or strict mapping.
            // ExitCode::from(u8) exists.

            if code == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        },
    }
}
