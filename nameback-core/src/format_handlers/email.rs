#![allow(unused_assignments)]

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Email metadata extracted from .eml files
#[derive(Debug, Clone)]
pub struct EmailMetadata {
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date: Option<String>,
}

/// Extracts metadata from .eml (RFC 822) email files
pub fn extract_email_metadata(path: &Path) -> Result<EmailMetadata> {
    let file = File::open(path).context("Failed to open email file")?;
    let reader = BufReader::new(file);

    let mut subject = None;
    let mut from = None;
    let mut date = None;
    let mut in_headers = true;

    for line in reader.lines().take(200) {
        let line = line?;

        // Empty line marks end of headers
        if line.trim().is_empty() && in_headers {
            in_headers = false;
            break;
        }

        if !in_headers {
            break;
        }

        // Parse headers (case-insensitive)
        let lower_line = line.to_lowercase();

        if lower_line.starts_with("subject:") {
            subject = Some(extract_header_value(&line, "subject:"));
        } else if lower_line.starts_with("from:") {
            from = Some(extract_header_value(&line, "from:"));
        } else if lower_line.starts_with("date:") {
            date = Some(extract_header_value(&line, "date:"));
        }
    }

    Ok(EmailMetadata {
        subject,
        from,
        date,
    })
}

/// Extracts the value part of a header line
fn extract_header_value(line: &str, prefix: &str) -> String {
    line[prefix.len()..]
        .trim()
        .to_string()
}

/// Formats email metadata into a filename
/// Format: "Subject_from_Sender_YYYY-MM-DD"
pub fn format_email_filename(metadata: &EmailMetadata) -> Option<String> {
    let mut parts = Vec::new();

    // Add subject
    if let Some(subject) = &metadata.subject {
        let cleaned = clean_email_field(subject);
        if !cleaned.is_empty() {
            parts.push(cleaned);
        }
    }

    // Add sender (extract name from email)
    if let Some(from) = &metadata.from {
        let sender = extract_sender_name(from);
        if !sender.is_empty() {
            parts.push(format!("from_{}", sender));
        }
    }

    // Add date (extract just YYYY-MM-DD)
    if let Some(date) = &metadata.date {
        if let Some(simple_date) = extract_simple_date(date) {
            parts.push(simple_date);
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("_"))
    }
}

/// Cleans email field for use in filename
fn clean_email_field(field: &str) -> String {
    field
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// Extracts name from email address
/// Examples: "John Doe <john@example.com>" -> "John_Doe"
///           "john@example.com" -> "john"
fn extract_sender_name(from: &str) -> String {
    // Check for "Name <email>" format
    if let Some(angle_pos) = from.find('<') {
        let name = from[..angle_pos].trim();
        if !name.is_empty() {
            return clean_email_field(name);
        }
    }

    // Extract username from email address
    if let Some(at_pos) = from.find('@') {
        let username = from[..at_pos].trim();
        return clean_email_field(username);
    }

    // Fallback: use as-is
    clean_email_field(from)
}

/// Extracts simple date (YYYY-MM-DD) from email date header
/// Email dates are in RFC 2822 format: "Mon, 15 Oct 2023 14:30:22 +0000"
fn extract_simple_date(date_str: &str) -> Option<String> {
    // Look for date pattern: DD Mon YYYY or YYYY-MM-DD
    let parts: Vec<&str> = date_str.split_whitespace().collect();

    // Try to find year (4 digits)
    let year = parts.iter().find(|s| {
        s.len() == 4 && s.chars().all(|c| c.is_numeric())
    })?;

    // Try to find month (3-letter abbreviation or 2-digit)
    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let month = parts.iter().find_map(|s| {
        month_names.iter().position(|&m| s.contains(m))
            .map(|i| format!("{:02}", i + 1))
    })?;

    // Try to find day (1-2 digits)
    let day = parts.iter().find_map(|s| {
        if s.len() <= 2 && s.chars().all(|c| c.is_numeric()) {
            let d: u32 = s.parse().ok()?;
            if (1..=31).contains(&d) {
                return Some(format!("{:02}", d));
            }
        }
        None
    })?;

    Some(format!("{}-{}-{}", year, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_email_field() {
        assert_eq!(clean_email_field("Meeting Notes"), "Meeting_Notes");
        assert_eq!(clean_email_field("Re: Project Update"), "Re_Project_Update");
        assert_eq!(clean_email_field("Test!!!"), "Test");
    }

    #[test]
    fn test_extract_sender_name() {
        assert_eq!(
            extract_sender_name("John Doe <john@example.com>"),
            "John_Doe"
        );
        assert_eq!(extract_sender_name("john@example.com"), "john");
        assert_eq!(
            extract_sender_name("jane.smith@example.com"),
            "janesmith"  // periods get filtered out by clean_email_field
        );
    }

    #[test]
    fn test_extract_simple_date() {
        let date = "Mon, 15 Oct 2023 14:30:22 +0000";
        assert_eq!(extract_simple_date(date), Some("2023-10-15".to_string()));

        let date2 = "15 Oct 2023 14:30:22";
        assert_eq!(extract_simple_date(date2), Some("2023-10-15".to_string()));
    }

    #[test]
    fn test_format_email_filename() {
        let metadata = EmailMetadata {
            subject: Some("Weekly Status Report".to_string()),
            from: Some("Jane Smith <jane@example.com>".to_string()),
            date: Some("Mon, 15 Oct 2023 14:30:22 +0000".to_string()),
        };

        let result = format_email_filename(&metadata);
        assert!(result.is_some());
        let filename = result.unwrap();
        assert!(filename.contains("Weekly_Status_Report"));
        assert!(filename.contains("from_Jane_Smith"));
        assert!(filename.contains("2023-10-15"));
    }

    #[test]
    fn test_format_email_filename_minimal() {
        let metadata = EmailMetadata {
            subject: Some("Test".to_string()),
            from: None,
            date: None,
        };

        let result = format_email_filename(&metadata);
        assert_eq!(result, Some("Test".to_string()));
    }
}
