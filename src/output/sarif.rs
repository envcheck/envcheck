use std::io::{self, Write};

use serde::Serialize;

use crate::rules::Diagnostic;

/// SARIF v2.1.0 output for GitHub Security tab integration
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifReport {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    pub driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    pub name: String,
    pub version: String,
    pub information_uri: String,
    pub rules: Vec<SarifRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    pub short_description: SarifMessage,
    pub default_configuration: SarifDefaultConfiguration,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifMessage {
    pub text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDefaultConfiguration {
    pub level: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    pub rule_id: String,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    pub region: SarifRegion,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    pub uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: usize,
}

pub fn write_sarif(diagnostics: &[Diagnostic], writer: &mut dyn Write) -> io::Result<()> {
    let results: Vec<SarifResult> = diagnostics
        .iter()
        .map(|d| SarifResult {
            rule_id: d.id.to_string(),
            level: match d.severity {
                crate::rules::Severity::Error => "error".to_string(),
                crate::rules::Severity::Warning => "warning".to_string(),
                crate::rules::Severity::Info => "note".to_string(),
            },
            message: SarifMessage {
                text: d.message.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: d.path.display().to_string(),
                    },
                    region: SarifRegion {
                        start_line: d.line.unwrap_or(1),
                    },
                },
            }],
        })
        .collect();

    let report = SarifReport {
        schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "envcheck".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    information_uri: "https://github.com/envcheck/envcheck".to_string(),
                    rules: vec![
                        SarifRule {
                            id: "E001".to_string(),
                            name: "DuplicateKey".to_string(),
                            short_description: SarifMessage {
                                text: "Duplicate key detected".to_string(),
                            },
                            default_configuration: SarifDefaultConfiguration {
                                level: "error".to_string(),
                            },
                        },
                        SarifRule {
                            id: "E002".to_string(),
                            name: "InvalidSyntax".to_string(),
                            short_description: SarifMessage {
                                text: "Invalid syntax".to_string(),
                            },
                            default_configuration: SarifDefaultConfiguration {
                                level: "error".to_string(),
                            },
                        },
                        SarifRule {
                            id: "W001".to_string(),
                            name: "EmptyValue".to_string(),
                            short_description: SarifMessage {
                                text: "Empty value".to_string(),
                            },
                            default_configuration: SarifDefaultConfiguration {
                                level: "warning".to_string(),
                            },
                        },
                        SarifRule {
                            id: "W002".to_string(),
                            name: "TrailingWhitespace".to_string(),
                            short_description: SarifMessage {
                                text: "Trailing whitespace".to_string(),
                            },
                            default_configuration: SarifDefaultConfiguration {
                                level: "warning".to_string(),
                            },
                        },
                    ],
                },
            },
            results,
        }],
    };

    let json = serde_json::to_string_pretty(&report)?;
    writeln!(writer, "{json}")
}
