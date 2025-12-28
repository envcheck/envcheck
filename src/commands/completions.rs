use std::io;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::error::Result;

pub fn run(shell: Shell) -> Result<()> {
    use crate::commands::Commands;

    // Wrapper struct to access CommandFactory for the Commands enum
    #[derive(clap::Parser)]
    #[command(name = "envcheck")]
    #[command(about = "A fast, modern CLI for linting .env files")]
    struct CliForCompletion {
        #[command(subcommand)]
        command: Commands,
    }

    let mut cmd = CliForCompletion::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
    Ok(())
}
