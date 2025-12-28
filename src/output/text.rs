use colored::*;
use std::io::Write;

use crate::output::OutputFormatter;
use crate::rules::{Diagnostic, Severity};

pub struct TextFormatter;

impl OutputFormatter for TextFormatter {
    fn write(&self, diagnostics: &[Diagnostic], writer: &mut dyn Write) -> std::io::Result<()> {
        if diagnostics.is_empty() {
            return Ok(());
        }

        // Group by file for cleaner output
        // Note: Assuming diagnostics are sorted by file, but we can't assume that actually.
        // Let's print sequentially for now, simpler.

        for diagnostic in diagnostics {
            let severity_str = match diagnostic.severity {
                Severity::Error => "error".red().bold(),
                Severity::Warning => "warning".yellow().bold(),
                Severity::Info => "info".blue().bold(),
            };

            let code = diagnostic.id.to_string().white().bold();

            // Format: error[E001]: message
            writeln!(writer, "{}[{}]: {}", severity_str, code, diagnostic.message)?;

            // Location:  --> file:line
            let path_str = diagnostic.path.to_string_lossy();
            if let Some(line) = diagnostic.line {
                writeln!(writer, "  {} {}:{}", "-->".blue(), path_str, line)?;
            } else {
                writeln!(writer, "  {} {}", "-->".blue(), path_str)?;
            }

            writeln!(writer)?;
        }

        Ok(())
    }
}
