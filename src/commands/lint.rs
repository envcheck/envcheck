use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::error::Result;
use crate::output::{write_diagnostics, Format};
use crate::parser::EnvFile;
use crate::rules::{check_file, Diagnostic, Severity};

pub fn run(files: &[PathBuf], format: Format) -> Result<()> {
    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    // Process files in parallel
    let results: Vec<_> = files
        .par_iter()
        .map(|path| match EnvFile::parse(path) {
            Ok(env_file) => {
                let diagnostics = check_file(&env_file);
                for d in &diagnostics {
                    match d.severity {
                        Severity::Error => {
                            error_count.fetch_add(1, Ordering::Relaxed);
                        },
                        Severity::Warning => {
                            warning_count.fetch_add(1, Ordering::Relaxed);
                        },
                        _ => {},
                    }
                }
                Ok(diagnostics)
            },
            Err(e) => Err(e),
        })
        .collect();

    // Flatten results, propagating first error if any
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();
    for result in results {
        match result {
            Ok(diagnostics) => all_diagnostics.extend(diagnostics),
            Err(e) => return Err(e),
        }
    }

    // Sort all diagnostics by File path then Line
    all_diagnostics.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_diagnostics(format, &all_diagnostics, &mut handle)
        .map_err(|e| crate::error::EnvCheckError::read_error("stdout", e))?;

    if error_count.load(Ordering::Relaxed) > 0 {
        Err(crate::error::EnvCheckError::LintFailed {
            error_count: error_count.load(Ordering::Relaxed),
            warning_count: warning_count.load(Ordering::Relaxed),
        })
    } else {
        Ok(())
    }
}
