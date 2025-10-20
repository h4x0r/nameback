use regex::Regex;
use std::collections::HashSet;
use std::ffi::OsStr;

/// Generates a sanitized filename from a candidate name
pub fn generate_filename(
    candidate: &str,
    original_extension: Option<&OsStr>,
    existing_names: &mut HashSet<String>,
) -> String {
    // Sanitize the candidate name
    let sanitized = sanitize_filename(candidate);

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

/// Sanitizes a filename by removing or replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    // Replace problematic characters with underscores
    let re = Regex::new(r#"[/\\:*?"<>|]"#).unwrap();
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
