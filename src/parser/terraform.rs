use crate::error::{EnvCheckError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformVariable {
    pub name: String,
    pub path: PathBuf,
}

pub fn parse_directory(dir: &Path) -> Result<Vec<TerraformVariable>> {
    let mut variables = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "tf") {
            let content =
                fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;

            // hcl-rs can parse the body
            // We want to find blocks of type "variable"

            match hcl::parse(&content) {
                Ok(body) => {
                    for block in body.blocks() {
                        if block.identifier() == "variable" {
                            // Variable name is the first label
                            if let Some(label) = block.labels().first() {
                                let name = match label {
                                    hcl::BlockLabel::String(s) => s.clone(),
                                    hcl::BlockLabel::Identifier(s) => s.to_string(),
                                };

                                variables.push(TerraformVariable {
                                    name,
                                    path: path.to_path_buf(),
                                });
                            }
                        }
                    }
                },
                Err(e) => {
                    // Log error but don't stop? Or return error?
                    // For now, let's just create a new ParseError type for HCL or reuse existing
                    return Err(EnvCheckError::parse_error(path, 0, e.to_string()));
                },
            }
        }
    }

    // Deduplicate? Variables defined multiple times is error in Terraform, but maybe okay here.
    Ok(variables)
}
