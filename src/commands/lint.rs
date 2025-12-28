use std::io;
use std::path::PathBuf;

use crate::error::Result;
use crate::output::{write_diagnostics, Format};
use crate::parser::EnvFile;
use crate::rules::{check_file, Severity};

pub fn run(files: &[PathBuf], format: Format) -> Result<()> {
    let mut all_diagnostics = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    for path in files {
        // Parse the file
        // We accumulate errors rather than failing early if possible,
        // but for read errors we might want to warn and continue?
        // Let's just try to parse. If it fails (e.g. read error), we report it.
        // Wait, EnvFile::parse returns Result.
        match EnvFile::parse(path) {
            Ok(env_file) => {
                let diagnostics = check_file(&env_file);
                for d in &diagnostics {
                    match d.severity {
                        Severity::Error => error_count += 1,
                        Severity::Warning => warning_count += 1,
                        _ => {},
                    }
                }
                all_diagnostics.extend(diagnostics);
            },
            Err(e) => {
                // Return early on parse/io error for a specific file?
                // Or maybe create a generic "FileReadError" diagnostic?
                // The requirements say "exit code 1 on lint errors".
                // If we can't read a file, that's likely an error.
                return Err(e);
            },
        }
    }

    // Sort all diagnostics by File path then Line
    all_diagnostics.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_diagnostics(format, &all_diagnostics, &mut handle)
        .map_err(|e| crate::error::EnvCheckError::read_error("stdout", e))?;

    if error_count > 0 {
        Err(crate::error::EnvCheckError::LintFailed {
            error_count,
            warning_count,
        })
    } else {
        Ok(())
    }
}
