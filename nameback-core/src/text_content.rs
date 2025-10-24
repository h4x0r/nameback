use anyhow::Result;
use log::debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Extracts meaningful content from text-based files (txt, csv, md, etc.)
/// Returns the first useful portion suitable for a filename
pub fn extract_text_content(path: &Path) -> Result<Option<String>> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("md") | Some("markdown") => extract_from_markdown(path),
        Some("csv") => extract_from_csv(path),
        Some("txt") | Some("text") => extract_from_plain_text(path),
        Some("json") => extract_from_json(path),
        Some("yaml") | Some("yml") => extract_from_yaml(path),
        _ => extract_from_plain_text(path), // Default fallback
    }
}

/// Extracts first heading from markdown file
fn extract_from_markdown(path: &Path) -> Result<Option<String>> {
    debug!("Attempting to extract markdown heading from: {}", path.display());

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut in_frontmatter = false;
    let mut first_line_checked = false;

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        // Check first line for frontmatter start
        if !first_line_checked {
            first_line_checked = true;
            if trimmed == "---" {
                in_frontmatter = true;
                continue;
            }
        }

        // Track frontmatter boundaries
        if trimmed == "---" && in_frontmatter {
            in_frontmatter = false;
            continue;
        }

        // Parse frontmatter for title
        if in_frontmatter {
            if let Some(title_value) = trimmed.strip_prefix("title:") {
                let cleaned = title_value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim();
                if !cleaned.is_empty() && cleaned.len() > 3 {
                    let truncated = truncate_text(cleaned, 80);
                    debug!("Extracted markdown frontmatter title: {}", truncated);
                    return Ok(Some(truncated));
                }
            }
            continue;
        }

        // Look for markdown headers (# Header) after frontmatter
        if let Some(header) = trimmed.strip_prefix('#') {
            let cleaned = header.trim_start_matches('#').trim();

            // Skip generic headings
            let lower_cleaned = cleaned.to_lowercase();
            if is_generic_heading(&lower_cleaned) {
                continue;
            }

            if !cleaned.is_empty() && cleaned.len() > 3 {
                let truncated = truncate_text(cleaned, 80);
                debug!("Extracted markdown heading: {}", truncated);
                return Ok(Some(truncated));
            }
        }
    }

    // Fallback to first non-empty line if no heading found
    extract_from_plain_text(path)
}

/// Checks if a heading is too generic to be useful
fn is_generic_heading(heading: &str) -> bool {
    let generic_headings = [
        "introduction",
        "overview",
        "table of contents",
        "contents",
        "summary",
        "conclusion",
        "abstract",
        "preface",
        "foreword",
    ];

    generic_headings.contains(&heading)
}

/// Extracts header row or first data row from CSV file
fn extract_from_csv(path: &Path) -> Result<Option<String>> {
    debug!("Attempting to extract CSV headers from: {}", path.display());

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines_iter = reader.lines();

    // Get first line (headers)
    if let Some(Ok(header_line)) = lines_iter.next() {
        let headers: Vec<&str> = header_line
            .split(',')
            .map(|h| h.trim().trim_matches('"').trim_matches('\''))
            .collect();

        // Read first data row once (for numeric validation)
        let first_data_row = lines_iter.next();
        let data_values: Vec<&str> = match &first_data_row {
            Some(Ok(row)) => row.split(',').map(|v| v.trim().trim_matches('"')).collect(),
            Some(Err(e)) => {
                debug!("Failed to read CSV data row: {}", e);
                Vec::new()
            }
            None => Vec::new(),
        };

        // Find meaningful column names
        let mut meaningful_cols = Vec::new();

        for (idx, header) in headers.iter().enumerate() {
            // Skip if column is empty
            if header.is_empty() {
                continue;
            }

            let lower = header.to_lowercase();

            // Prioritize semantic columns
            let is_semantic = matches!(
                lower.as_str(),
                "name" | "title" | "description" | "subject" | "label" | "product" | "item"
            );

            // Skip identifier columns
            let is_id = lower.contains("id")
                || lower == "index"
                || lower.contains("key")
                || lower.contains("guid");

            // Skip timestamp columns
            let is_timestamp = lower.contains("date")
                || lower.contains("time")
                || lower.contains("created")
                || lower.contains("modified")
                || lower == "timestamp";

            if is_semantic {
                // Semantic columns go first
                meaningful_cols.insert(0, *header);
            } else if !is_id && !is_timestamp && meaningful_cols.len() < 2 {
                // Check if it's likely numeric by looking at first data row
                if let Some(value) = data_values.get(idx) {
                    // Skip if column data is numeric
                    if value.parse::<f64>().is_err() {
                        meaningful_cols.push(*header);
                    }
                }
            }

            // Limit to 2 columns max
            if meaningful_cols.len() >= 2 {
                break;
            }
        }

        if !meaningful_cols.is_empty() {
            let name = meaningful_cols.join("_");
            let cleaned = clean_text(&name);

            if cleaned.len() > 3 {
                let truncated = truncate_text(&cleaned, 80);
                debug!("Extracted CSV columns: {}", truncated);
                return Ok(Some(truncated));
            }
        }
    }

    Ok(None)
}

/// Extracts first meaningful line from plain text file
fn extract_from_plain_text(path: &Path) -> Result<Option<String>> {
    debug!("Attempting to extract text from: {}", path.display());

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read first 100 lines to gather enough text for key phrase extraction
    let mut full_text = String::new();
    let mut line_count = 0;

    for line in reader.lines().take(100) {
        let line = line?;
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            full_text.push_str(trimmed);
            full_text.push(' ');
            line_count += 1;
        }

        // Break early if we have enough text
        if full_text.len() > 500 {
            break;
        }
    }

    if full_text.len() > 10 {
        let cleaned = clean_text(&full_text);

        // Use key phrase extraction for longer text
        if cleaned.len() > 150 && line_count > 3 {
            debug!("Extracting key phrases from text file ({} chars, {} lines)", cleaned.len(), line_count);
            let phrases = crate::key_phrases::extract_key_phrases(&cleaned, 3);
            if !phrases.is_empty() {
                let best_phrase = &phrases[0];
                debug!("Selected key phrase from text: {}", best_phrase);
                return Ok(Some(best_phrase.clone()));
            }
        }

        // For shorter text or if key phrase extraction failed, truncate
        let truncated = truncate_text(&cleaned, 80);
        debug!("Extracted text content: {}", truncated);
        return Ok(Some(truncated));
    }

    Ok(None)
}

/// Attempts to extract title/name field from JSON file
fn extract_from_json(path: &Path) -> Result<Option<String>> {
    debug!("Attempting to extract from JSON: {}", path.display());

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read a reasonable chunk (first 4KB for nested search)
    let mut content = String::new();
    for line in reader.lines().take(100) {
        let line = line?;
        content.push_str(&line);
        content.push('\n');
        if content.len() > 4096 {
            break;
        }
    }

    // Try to parse as JSON and perform deep search
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        // Prioritized field paths to search
        let field_paths = vec![
            vec!["title"],
            vec!["name"],
            vec!["displayName"],
            vec!["label"],
            vec!["description"],
            vec!["metadata", "title"],
            vec!["data", "title"],
            vec!["data", "name"],
            vec!["config", "name"],
            vec!["package", "name"],
            vec!["project", "name"],
        ];

        for path_components in &field_paths {
            if let Some(text) = search_json_path(&json, path_components) {
                let cleaned = clean_text(&text);
                if cleaned.len() > 3 {
                    let truncated = truncate_text(&cleaned, 80);
                    debug!(
                        "Extracted JSON {}: {}",
                        path_components.join("."),
                        truncated
                    );
                    return Ok(Some(truncated));
                }
            }
        }
    }

    // Fallback to first meaningful text
    extract_from_plain_text(path)
}

/// Recursively searches JSON for a nested field path
fn search_json_path(json: &serde_json::Value, path: &[&str]) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let mut current = json;

    for &component in path {
        match current.get(component) {
            Some(value) => current = value,
            None => return None,
        }
    }

    // Extract string value
    current.as_str().map(|s| s.to_string())
}

/// Attempts to extract title/name field from YAML file
fn extract_from_yaml(path: &Path) -> Result<Option<String>> {
    debug!("Attempting to extract from YAML: {}", path.display());

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Simple line-by-line parsing for common title fields
    let title_patterns = ["title:", "name:", "description:", "label:"];

    for line in reader.lines().take(50) {
        let line = line?;
        let trimmed = line.trim();

        for pattern in &title_patterns {
            if let Some(value) = trimmed.strip_prefix(pattern) {
                let cleaned = clean_text(value.trim().trim_matches('"').trim_matches('\''));
                if cleaned.len() > 3 {
                    let truncated = truncate_text(&cleaned, 80);
                    debug!("Extracted YAML field: {}", truncated);
                    return Ok(Some(truncated));
                }
            }
        }
    }

    // Fallback to first meaningful line
    extract_from_plain_text(path)
}

/// Cleans text for use in filenames (similar to pdf_content clean_text)
fn clean_text(text: &str) -> String {
    // Remove excessive whitespace and newlines
    let cleaned = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Collapse multiple spaces
    let mut result = String::new();
    let mut last_was_space = false;

    for ch in cleaned.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Truncates text to specified length, trying to break at word boundaries
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Try to break at last space before max_len
    let truncated = &text[..max_len];
    if let Some(last_space) = truncated.rfind(' ') {
        if last_space > max_len / 2 {
            return truncated[..last_space].to_string();
        }
    }

    // No good break point, just truncate
    truncated.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Title   with\n\nmultiple   spaces";
        let expected = "Title with multiple spaces";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_truncate_text_short() {
        let input = "Short text";
        assert_eq!(truncate_text(input, 80), "Short text");
    }

    #[test]
    fn test_truncate_text_long() {
        let input = "This is a very long text that needs to be truncated at a reasonable point in the middle of the sentence";
        let result = truncate_text(input, 50);
        assert!(result.len() <= 50);
        // The truncation logic finds last space before max_len
        // "This is a very long text that needs to be" (43 chars) ends at "be"
        assert!(result.len() < 50);
        assert!(result.len() > 25); // At least half the max length
    }
}
