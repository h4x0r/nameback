use anyhow::{Context, Result};
use std::path::Path;

/// Extracts text content from a PDF file and returns the first useful portion
pub fn extract_pdf_content(path: &Path) -> Result<Option<String>> {
    // Extract text from PDF
    let text = pdf_extract::extract_text(path)
        .context("Failed to extract text from PDF")?;

    // Clean up the text: remove extra whitespace, newlines, etc.
    let cleaned = clean_text(&text);

    // Take first 80 characters as potential filename
    if cleaned.len() > 10 {
        let truncated = if cleaned.len() > 80 {
            &cleaned[..80]
        } else {
            &cleaned
        };

        Ok(Some(truncated.to_string()))
    } else {
        // Not enough meaningful text
        Ok(None)
    }
}

/// Cleans extracted text for use in filenames
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Annual   Report\n\n2024\n  Financial  Statement";
        let expected = "Annual Report 2024 Financial Statement";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_clean_text_empty() {
        let input = "\n\n   \n  ";
        assert_eq!(clean_text(input), "");
    }
}
