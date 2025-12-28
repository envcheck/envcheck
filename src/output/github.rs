use std::io::Write;

use crate::output::OutputFormatter;
use crate::rules::{Diagnostic, Severity};

pub struct GithubFormatter;

impl OutputFormatter for GithubFormatter {
    fn write(&self, diagnostics: &[Diagnostic], writer: &mut dyn Write) -> std::io::Result<()> {
        for diagnostic in diagnostics {
            let command = match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "notice",
            };

            // GitHub Actions format: ::command file={name},line={line}::{message}
            // See: https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-error-message

            let path = diagnostic.path.to_string_lossy();
            let line_part = if let Some(line) = diagnostic.line {
                format!("line={line},")
            } else {
                String::new()
            };

            // Escape message data
            // % -> %25, \r -> %0D, \n -> %0A
            let message = diagnostic
                .message
                .replace('%', "%25")
                .replace('\r', "%0D")
                .replace('\n', "%0A");

            writeln!(
                writer,
                "::{} file={},{}title={}::{}",
                command, path, line_part, diagnostic.id, message
            )?;
        }

        Ok(())
    }
}
