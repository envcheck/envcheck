use crate::error::Result;
use crate::output::Format;

/// Runs `lint` on all .env files in the current directory.
pub fn run(format: Format) -> Result<()> {
    // Glob for .env*
    let patterns = [".env", ".env.*"];
    let mut files = Vec::new();

    for pattern in patterns {
        for path in glob::glob(pattern)
            .map_err(|e| crate::error::EnvCheckError::GlobError {
                pattern: pattern.to_string(),
                source: e,
            })?
            .flatten()
        {
            if path.is_file() {
                files.push(path);
            }
        }
    }

    if files.is_empty() {
        println!("No .env files found in current directory.");
        return Ok(());
    }

    println!("Running doctor on {} files...", files.len());
    crate::commands::lint::run(&files, format)
}
