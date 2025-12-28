use crate::error::{EnvCheckError, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct EnvEntry {
    /// Comments preceding the key (or file header comments)
    comments: Vec<String>,
    /// The actual raw line containing the key (trimmed of whitespace, but keeping content)
    /// If None, this is just a standalone comment block (like a header)
    key_line: Option<String>,
    /// The extracted key for sorting
    key: Option<String>,
    #[allow(dead_code)]
    /// Original line (allows detecting blank lines to preserve spacing if needed)
    original_lines: Vec<String>,
}

pub fn run(files: &[PathBuf]) -> Result<()> {
    for path in files {
        fix_file(path)?;
    }
    Ok(())
}

fn fix_file(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path).map_err(|e| EnvCheckError::read_error(path, e))?;

    if content.trim().is_empty() {
        return Ok(());
    }

    let entries = parse_preserving(path, &content);

    let mut header = Vec::new();
    let mut footer = Vec::new();
    let mut body = Vec::new();

    let mut in_header = true;

    for entry in entries {
        if entry.key.is_some() {
            in_header = false;
            body.push(entry);
        } else if in_header {
            header.push(entry);
        } else {
            // If we are not in header, and hit a no-key, it's either footer or intermediate.
            // Treat as footer for now.
            footer.push(entry);
        }
    }

    // Sort body
    body.sort_by(|a, b| a.key.cmp(&b.key));

    let mut final_output = String::new();

    for e in header {
        final_output.push_str(&render_entry(&e));
    }

    for e in body {
        final_output.push_str(&render_entry(&e));
    }

    for e in footer {
        final_output.push_str(&render_entry(&e));
    }

    // Usually we want one final newline.
    if !final_output.ends_with('\n') {
        final_output.push('\n');
    }

    fs::write(path, final_output).map_err(|e| EnvCheckError::write_error(path, e))?;

    Ok(())
}

fn parse_preserving(_path: &Path, content: &str) -> Vec<EnvEntry> {
    let mut entries = Vec::new();
    let mut current_comments = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Blank line.
            // If we have accumulated comments, flush them as a distinct block (e.g. Header).
            if !current_comments.is_empty() {
                entries.push(EnvEntry {
                    comments: current_comments.clone(),
                    key_line: None,
                    key: None,
                    original_lines: vec![],
                });
                current_comments.clear();
            }
            continue;
        }

        if trimmed.starts_with('#') {
            current_comments.push(String::from(trimmed));
            continue;
        }

        // It's a key line (or invalid garbage)
        // Check if key
        if let Some((k, _)) = trimmed.split_once('=') {
            let key_str = k.trim().trim_start_matches("export ").trim().to_string();
            // Valid-ish key.
            entries.push(EnvEntry {
                comments: current_comments.clone(),
                key_line: Some(trimmed.to_string()),
                key: Some(key_str),
                original_lines: vec![],
            });
            current_comments.clear();
        } else {
            // Garbage or complex line?
            // Treat as comment/blob to be safe?
            // If we treat as comment, it gets attached to next key.
            current_comments.push(String::from(trimmed));
        }
    }

    // EOF. If comments remain, add as Footer entry
    if !current_comments.is_empty() {
        entries.push(EnvEntry {
            comments: current_comments,
            key_line: None,
            key: None,
            original_lines: vec![],
        });
    }

    entries
}

fn render_entry(entry: &EnvEntry) -> String {
    let mut s = String::new();
    for comment in &entry.comments {
        s.push_str(comment);
        s.push('\n');
    }
    if let Some(ref line) = entry.key_line {
        s.push_str(line);
        s.push('\n');
    }
    s
}
