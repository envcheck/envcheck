use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

use crate::error::{EnvCheckError, Result};
use crate::output::{write_diagnostics, Format};
use crate::parser::EnvFile;
use crate::rules::{Diagnostic, RuleId, Severity};

pub fn run(files: &[PathBuf], format: Format) -> Result<()> {
    if files.len() < 2 {
        return Err(EnvCheckError::InsufficientFiles { count: files.len() });
    }

    // Parse all files
    let mut env_files = Vec::new();
    for path in files {
        let env_file = EnvFile::parse(path)?;
        env_files.push(env_file);
    }

    // The first file is the "reference" (e.g., .env.example)
    // Actually, usually we want to cross-compare everything, or use the first as base.
    // The requirement says: "compare .env.example .env.local .env.prod"
    // "Key missing in comparison file" (W004)
    // Implicitly, usually .env.example is the source of truth.
    // Let's implement: unique union of all keys, or just use File[0] as Set A?
    // "Compare keys across files" often means checks consistency.
    // Let's take the UNION of all keys, OR just rely on File[0] as the strict definition?
    //envdoc-go documentation says it finds "Missing keys across files".
    // Let's check: For every file A, and every other file B, are there keys in A missing in B?
    // Or simpler: Use File[0] as the "reference". Any key in File[0] MUST exist in File[1..N].
    // Also, keys in File[1..N] that are NOT in File[0] might be "Extra keys" (Info/Warning?).
    // Let's explicitly check: Reference vs Others.

    let reference = &env_files[0];
    let others = &env_files[1..];

    let ref_keys: HashSet<&String> = reference.vars.iter().map(|v| &v.key).collect();

    let mut diagnostics = Vec::new();

    for other in others {
        let other_keys: HashSet<&String> = other.vars.iter().map(|v| &v.key).collect();

        // Check for keys in Reference missing in Other (W004)
        for &key in &ref_keys {
            if !other_keys.contains(key) {
                diagnostics.push(Diagnostic {
                    id: RuleId::W004,
                    severity: Severity::Warning,
                    message: format!(
                        "Missing key '{}' (present in {})",
                        key,
                        reference.path.display()
                    ),
                    path: other.path.clone(),
                    line: None, // We don't have a line number for a missing key
                });
            }
        }

        // Optional: Check for keys in Other missing in Reference (Reverse check)
        // Usually local envs might have extra secrets not in example? Or example should list everything?
        // Let's add it as Info or specialized Warning?
        // Requirement table says: `W004 | Key missing in comparison file | Warning`
        // Let's stick to Reference -> Other direction as primary warning.
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write_diagnostics(format, &diagnostics, &mut handle)
        .map_err(|e| EnvCheckError::read_error("stdout", e))?;

    // If warnings found, maybe exit non-zero?
    // "Lint failed" error usually just checks error_count > 0.
    // But compare is utility. Let's return Ok, unless we strictly want to fail CI.
    // Our Error types handle failure.
    // If diagnostics exist, we might want to return `Ok` but the user sees output.
    // Only return Error if execution failed.

    Ok(())
}
