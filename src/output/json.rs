use serde::Serialize;
use std::io::Write;

use crate::output::OutputFormatter;
use crate::rules::Diagnostic;

#[derive(Serialize)]
struct JsonDiagnostic {
    rule: String,
    severity: String,
    message: String,
    file: String,
    line: Option<usize>,
}

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn write(&self, diagnostics: &[Diagnostic], writer: &mut dyn Write) -> std::io::Result<()> {
        let json_diagnostics: Vec<JsonDiagnostic> = diagnostics
            .iter()
            .map(|d| JsonDiagnostic {
                rule: d.id.to_string(),
                severity: d.severity.to_string().to_lowercase(),
                message: d.message.clone(),
                file: d.path.to_string_lossy().to_string(),
                line: d.line,
            })
            .collect();

        let json =
            serde_json::to_string_pretty(&json_diagnostics).map_err(std::io::Error::other)?;

        writeln!(writer, "{json}")
    }
}
