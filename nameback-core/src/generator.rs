use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;
use crate::extractor::FileMetadata;

/// Generates a sanitized filename from a candidate name
#[allow(dead_code)]
pub fn generate_filename(
    candidate: &str,
    original_extension: Option<&OsStr>,
    existing_names: &mut HashSet<String>,
) -> String {
    generate_filename_with_metadata(candidate, original_extension, existing_names, None)
}

/// Generates a sanitized filename from a candidate name with optional metadata enhancements
pub fn generate_filename_with_metadata(
    candidate: &str,
    original_extension: Option<&OsStr>,
    existing_names: &mut HashSet<String>,
    metadata: Option<&FileMetadata>,
) -> String {
    // Sanitize the candidate name
    let mut sanitized = sanitize_filename(candidate);

    // Add location and timestamp if enabled in config and available in metadata
    if let Some(meta) = metadata {
        let mut additions: Vec<String> = Vec::new();

        // Add GPS location if enabled and available
        if meta.include_location {
            if let Some(location) = &meta.gps_location {
                // Try geocoding first (enabled by default)
                // This will convert GPS to city names like "Seattle_WA"
                let location_str = if meta.geocode_enabled.unwrap_or(true) {
                    // Try to reverse geocode to get city/state name
                    crate::geocoding::reverse_geocode(location.latitude, location.longitude)
                        .unwrap_or_else(|| {
                            // Fall back to coordinates if geocoding fails
                            crate::location_timestamp::format_location(location)
                        })
                } else {
                    // User disabled geocoding, use coordinates format
                    crate::location_timestamp::format_location(location)
                };
                additions.push(location_str);
            }
        }

        // Add timestamp if enabled and available (use date_time_original or creation_date)
        if meta.include_timestamp {
            if let Some(timestamp) = meta.date_time_original.as_ref().or(meta.creation_date.as_ref()) {
                // Format timestamp to YYYY-MM-DD for filename
                if let Some(formatted) = format_timestamp_for_filename(timestamp) {
                    additions.push(formatted);
                }
            }
        }

        // Append additions to the filename
        if !additions.is_empty() {
            let additions_str = additions.join("_");
            sanitized = format!("{}_{}", sanitized, additions_str);
        }
    }

    // Limit length to 200 characters to leave room for extension and counter
    let mut base_name = sanitized.chars().take(200).collect::<String>();

    // If empty after sanitization, use a default
    if base_name.is_empty() {
        base_name = "renamed_file".to_string();
    }

    // Add extension if present
    let extension = original_extension
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    // Generate unique filename
    let mut filename = format!("{}{}", base_name, extension);
    let mut counter = 1;

    while existing_names.contains(&filename) {
        filename = format!("{}_{}{}", base_name, counter, extension);
        counter += 1;
    }

    existing_names.insert(filename.clone());
    filename
}

/// Formats a timestamp string for use in filename (YYYY-MM-DD format)
fn format_timestamp_for_filename(timestamp: &str) -> Option<String> {
    // Try to extract date in YYYY:MM:DD format from EXIF timestamp
    // Format is typically "YYYY:MM:DD HH:MM:SS"
    if let Some(date_part) = timestamp.split_whitespace().next() {
        // Convert colons to dashes for filename compatibility
        let formatted = date_part.replace(':', "-");
        if formatted.len() == 10 && formatted.chars().filter(|c| *c == '-').count() == 2 {
            return Some(formatted);
        }
    }
    None
}

/// Sanitizes a filename by removing or replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    // Replace problematic characters with underscores (includes parentheses for cleaner names)
    let re = Regex::new(r#"[/\\:*?"<>|()\[\]]"#).unwrap();
    let mut sanitized = re.replace_all(name, "_").to_string();

    // Replace spaces with underscores
    sanitized = sanitized.replace(' ', "_");

    // Remove control characters
    sanitized = sanitized.chars().filter(|c| !c.is_control()).collect();

    // Collapse multiple underscores into one
    let re_multiple = Regex::new(r"_{2,}").unwrap();
    sanitized = re_multiple.replace_all(&sanitized, "_").to_string();

    // Trim underscores from start and end
    sanitized.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("file:name*test"), "file_name_test");
        assert_eq!(sanitize_filename("___test___"), "test");
        assert_eq!(sanitize_filename("a/b\\c:d"), "a_b_c_d");
    }

    #[test]
    fn test_generate_filename_unique() {
        let mut existing = HashSet::new();

        let name1 = generate_filename("test", Some(OsStr::new("txt")), &mut existing);
        assert_eq!(name1, "test.txt");

        let name2 = generate_filename("test", Some(OsStr::new("txt")), &mut existing);
        assert_eq!(name2, "test_1.txt");

        let name3 = generate_filename("test", Some(OsStr::new("txt")), &mut existing);
        assert_eq!(name3, "test_2.txt");
    }
}
