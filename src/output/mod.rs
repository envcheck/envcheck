use crate::rules::Diagnostic;
use std::io::Write;

mod github;
mod json;
mod text;

pub use github::GithubFormatter;
pub use json::JsonFormatter;
pub use text::TextFormatter;

/// Supported output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
    Github,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "github" => Ok(Self::Github),
            _ => Err(format!("Unknown format: {s}")),
        }
    }
}

/// Trait for diagnostic formatters.
pub trait OutputFormatter {
    /// Writes the diagnostics to the writer.
    fn write(&self, diagnostics: &[Diagnostic], writer: &mut dyn Write) -> std::io::Result<()>;
}

/// Writes diagnostics using the specified format.
pub fn write_diagnostics(
    format: Format,
    diagnostics: &[Diagnostic],
    writer: &mut dyn Write,
) -> std::io::Result<()> {
    match format {
        Format::Text => TextFormatter.write(diagnostics, writer),
        Format::Json => JsonFormatter.write(diagnostics, writer),
        Format::Github => GithubFormatter.write(diagnostics, writer),
    }
}
