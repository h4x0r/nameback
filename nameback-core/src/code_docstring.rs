use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Supported code languages for docstring extraction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Java,
    C,
    Cpp,
}

/// Detects programming language from file extension
pub fn detect_language(path: &Path) -> Option<CodeLanguage> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "py" => Some(CodeLanguage::Python),
        "js" => Some(CodeLanguage::JavaScript),
        "ts" => Some(CodeLanguage::TypeScript),
        "rs" => Some(CodeLanguage::Rust),
        "java" => Some(CodeLanguage::Java),
        "c" | "h" => Some(CodeLanguage::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(CodeLanguage::Cpp),
        _ => None,
    }
}

/// Extracts docstring from source code file
pub fn extract_docstring(path: &Path) -> Result<Option<String>> {
    let lang = match detect_language(path) {
        Some(l) => l,
        None => return Ok(None),
    };

    match lang {
        CodeLanguage::Python => extract_python_docstring(path),
        CodeLanguage::JavaScript | CodeLanguage::TypeScript => extract_jsdoc(path),
        CodeLanguage::Rust => extract_rust_doc(path),
        CodeLanguage::Java => extract_javadoc(path),
        CodeLanguage::C | CodeLanguage::Cpp => extract_doxygen(path),
    }
}

/// Extracts Python module-level docstring
/// Looks for first triple-quoted string (""" or ''')
fn extract_python_docstring(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut in_docstring = false;
    let mut docstring_delim = "";
    let mut docstring_lines = Vec::new();
    let mut found_docstring = false;

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Skip shebang and encoding declarations
        if trimmed.starts_with('#') && !found_docstring {
            continue;
        }

        // Skip empty lines before docstring
        if trimmed.is_empty() && !found_docstring && !in_docstring {
            continue;
        }

        // Check for docstring start
        if !in_docstring && (trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")) {
            docstring_delim = if trimmed.starts_with("\"\"\"") {
                "\"\"\""
            } else {
                "'''"
            };
            in_docstring = true;
            found_docstring = true;

            // Single-line docstring
            if trimmed.ends_with(docstring_delim) && trimmed.len() > 6 {
                let content = trimmed
                    .trim_start_matches(docstring_delim)
                    .trim_end_matches(docstring_delim)
                    .trim();
                if !content.is_empty() {
                    return Ok(Some(clean_docstring(content)));
                }
            } else {
                // Multi-line docstring
                let content = trimmed.trim_start_matches(docstring_delim).trim();
                if !content.is_empty() {
                    docstring_lines.push(content.to_string());
                }
            }
            continue;
        }

        // Inside docstring
        if in_docstring {
            if trimmed.ends_with(docstring_delim) {
                // End of docstring
                let content = trimmed.trim_end_matches(docstring_delim).trim();
                if !content.is_empty() {
                    docstring_lines.push(content.to_string());
                }
                break;
            } else {
                // Middle of docstring
                if !trimmed.is_empty() {
                    docstring_lines.push(trimmed.to_string());
                }
            }
        } else if found_docstring {
            // Found something else after checking for docstring
            break;
        }
    }

    if !docstring_lines.is_empty() {
        let combined = docstring_lines.join(" ");
        return Ok(Some(clean_docstring(&combined)));
    }

    Ok(None)
}

/// Extracts JSDoc @file or @module annotation
fn extract_jsdoc(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut in_block_comment = false;
    let mut comment_lines = Vec::new();

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Check for block comment start
        if trimmed.starts_with("/**") {
            in_block_comment = true;
            continue;
        }

        // Check for block comment end
        if in_block_comment && trimmed.ends_with("*/") {
            in_block_comment = false;
            break;
        }

        // Collect comment lines
        if in_block_comment {
            let cleaned = trimmed.trim_start_matches('*').trim();
            if !cleaned.is_empty() {
                comment_lines.push(cleaned.to_string());
            }
        }
    }

    // Look for @file or @module tags
    for line in &comment_lines {
        if let Some(content) = line.strip_prefix("@file ") {
            return Ok(Some(clean_docstring(content)));
        }
        if let Some(content) = line.strip_prefix("@module ") {
            return Ok(Some(clean_docstring(content)));
        }
    }

    // Fallback: use first meaningful comment line
    if let Some(first_line) = comment_lines.first() {
        if !first_line.starts_with('@') {
            return Ok(Some(clean_docstring(first_line)));
        }
    }

    Ok(None)
}

/// Extracts Rust module documentation (//! or /*! ... */)
fn extract_rust_doc(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut doc_lines = Vec::new();

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Module doc comment
        if let Some(content) = trimmed.strip_prefix("//!") {
            let cleaned = content.trim();
            if !cleaned.is_empty() {
                doc_lines.push(cleaned.to_string());
            }
        } else if !doc_lines.is_empty() {
            // Stop at first non-doc line
            break;
        }
    }

    if !doc_lines.is_empty() {
        let combined = doc_lines.join(" ");
        return Ok(Some(clean_docstring(&combined)));
    }

    Ok(None)
}

/// Extracts Javadoc from class or file header
fn extract_javadoc(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut in_block_comment = false;
    let mut comment_lines = Vec::new();

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Check for Javadoc start
        if trimmed.starts_with("/**") {
            in_block_comment = true;
            continue;
        }

        // Check for block comment end
        if in_block_comment && trimmed.ends_with("*/") {
            in_block_comment = false;
            break;
        }

        // Collect comment lines
        if in_block_comment {
            let cleaned = trimmed.trim_start_matches('*').trim();
            if !cleaned.is_empty() && !cleaned.starts_with('@') {
                comment_lines.push(cleaned.to_string());
            }
        }
    }

    if !comment_lines.is_empty() {
        let combined = comment_lines.join(" ");
        return Ok(Some(clean_docstring(&combined)));
    }

    Ok(None)
}

/// Extracts Doxygen-style comments
fn extract_doxygen(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut in_block_comment = false;
    let mut comment_lines = Vec::new();

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Check for Doxygen block start (/** or /*!)
        if trimmed.starts_with("/**") || trimmed.starts_with("/*!") {
            in_block_comment = true;
            continue;
        }

        // Check for block comment end
        if in_block_comment && trimmed.ends_with("*/") {
            in_block_comment = false;
            break;
        }

        // Collect comment lines
        if in_block_comment {
            let cleaned = trimmed.trim_start_matches('*').trim();
            if !cleaned.is_empty() && !cleaned.starts_with('@') && !cleaned.starts_with('\\') {
                comment_lines.push(cleaned.to_string());
            }
        }

        // Also check for /// or //! style
        if let Some(content) = trimmed.strip_prefix("///") {
            let cleaned = content.trim();
            if !cleaned.is_empty() {
                comment_lines.push(cleaned.to_string());
            }
        } else if !comment_lines.is_empty() && !in_block_comment {
            break;
        }
    }

    if !comment_lines.is_empty() {
        let combined = comment_lines.join(" ");
        return Ok(Some(clean_docstring(&combined)));
    }

    Ok(None)
}

/// Cleans and truncates docstring to reasonable length
fn clean_docstring(text: &str) -> String {
    // Remove excess whitespace
    let cleaned = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();

    // Truncate at first sentence or 100 chars
    if let Some(period_pos) = cleaned.find(". ") {
        if period_pos < 100 {
            return cleaned[..period_pos].to_string();
        }
    }

    if cleaned.len() > 100 {
        if let Some(space_pos) = cleaned[..100].rfind(' ') {
            return cleaned[..space_pos].to_string();
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            detect_language(Path::new("test.py")),
            Some(CodeLanguage::Python)
        );
        assert_eq!(
            detect_language(Path::new("test.js")),
            Some(CodeLanguage::JavaScript)
        );
        assert_eq!(
            detect_language(Path::new("test.rs")),
            Some(CodeLanguage::Rust)
        );
        assert_eq!(
            detect_language(Path::new("test.java")),
            Some(CodeLanguage::Java)
        );
        assert_eq!(detect_language(Path::new("test.txt")), None);
    }

    #[test]
    fn test_clean_docstring() {
        let input = "This is a test.   With  extra   spaces.";
        let result = clean_docstring(input);
        assert_eq!(result, "This is a test");
    }

    #[test]
    fn test_clean_docstring_truncate() {
        let input = "a ".repeat(60); // 120 chars
        let result = clean_docstring(&input);
        assert!(result.len() < 100);
    }
}
