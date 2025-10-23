use anyhow::Result;
use log::debug;
use std::path::Path;
use std::process::Command;

/// Inspects archive contents and suggests a name
/// Looks at the first significant file inside the archive
pub fn extract_archive_info(path: &Path) -> Result<Option<String>> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("zip") => inspect_zip(path),
        Some("tar") | Some("tgz") | Some("tar.gz") => inspect_tar(path),
        Some("7z") => inspect_7z(path),
        Some("rar") => inspect_rar(path),
        _ => Ok(None),
    }
}

/// Inspects ZIP archive
fn inspect_zip(path: &Path) -> Result<Option<String>> {
    debug!("Inspecting ZIP archive: {}", path.display());

    let output = Command::new("unzip")
        .arg("-l") // List contents
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let listing = String::from_utf8_lossy(&output.stdout);
            return extract_from_listing(&listing, "");
        }
    }

    Ok(None)
}

/// Inspects TAR archive
fn inspect_tar(path: &Path) -> Result<Option<String>> {
    debug!("Inspecting TAR archive: {}", path.display());

    let output = Command::new("tar")
        .arg("-tf") // List files
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let listing = String::from_utf8_lossy(&output.stdout);
            return extract_from_listing(&listing, "");
        }
    }

    Ok(None)
}

/// Inspects 7z archive
fn inspect_7z(path: &Path) -> Result<Option<String>> {
    debug!("Inspecting 7z archive: {}", path.display());

    let output = Command::new("7z")
        .arg("l") // List contents
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let listing = String::from_utf8_lossy(&output.stdout);
            return extract_from_listing(&listing, "");
        }
    }

    Ok(None)
}

/// Inspects RAR archive
fn inspect_rar(path: &Path) -> Result<Option<String>> {
    debug!("Inspecting RAR archive: {}", path.display());

    // Try unrar first, then fall back to rar
    let output = Command::new("unrar")
        .arg("l") // List contents
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let listing = String::from_utf8_lossy(&output.stdout);
            return extract_from_listing(&listing, "");
        }
    }

    Ok(None)
}

/// Extracts meaningful name from archive listing
fn extract_from_listing(listing: &str, _prefix: &str) -> Result<Option<String>> {
    let mut files = Vec::new();

    // Parse listing to find files (skip directories)
    for line in listing.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Extract filename (usually last field in listing)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(filename) = parts.last() {
            // Skip directories (end with /)
            if filename.ends_with('/') {
                continue;
            }

            // Skip common junk files
            if is_junk_file(filename) {
                continue;
            }

            files.push(filename.to_string());
        }
    }

    // If only one significant file, use its name
    if files.len() == 1 {
        if let Some(name) = extract_stem(&files[0]) {
            return Ok(Some(name));
        }
    }

    // If multiple files share a common prefix, use that
    if files.len() > 1 {
        if let Some(common) = find_common_prefix(&files) {
            if common.len() > 3 {
                return Ok(Some(common));
            }
        }
    }

    Ok(None)
}

/// Checks if a filename is a junk file to skip
fn is_junk_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();

    // System files
    if lower.starts_with(".ds_store")
        || lower.starts_with("thumbs.db")
        || lower.starts_with("desktop.ini")
        || lower.starts_with("__macosx")
    {
        return true;
    }

    // Common metadata files
    if lower.ends_with(".txt") && (lower.contains("readme") || lower.contains("license")) {
        return true;
    }

    false
}

/// Extracts meaningful stem from a filename
fn extract_stem(filename: &str) -> Option<String> {
    // Get just the filename without path
    let name = filename.split('/').next_back().unwrap_or(filename);

    // Remove extension
    let stem = if let Some(dot_pos) = name.rfind('.') {
        &name[..dot_pos]
    } else {
        name
    };

    if stem.len() > 2 {
        Some(clean_name(stem))
    } else {
        None
    }
}

/// Finds common prefix among filenames
fn find_common_prefix(files: &[String]) -> Option<String> {
    if files.is_empty() {
        return None;
    }

    let first = &files[0];
    let mut prefix_len = 0;

    'outer: for (i, ch) in first.chars().enumerate() {
        for file in files.iter().skip(1) {
            if file.chars().nth(i) != Some(ch) {
                break 'outer;
            }
        }
        prefix_len = i + 1;
    }

    if prefix_len > 3 {
        let prefix = &first[..prefix_len];
        // Trim at last separator
        if let Some(last_sep) = prefix.rfind(|c: char| !c.is_alphanumeric()) {
            if last_sep > 0 {
                return Some(clean_name(&prefix[..last_sep]));
            }
        }
        Some(clean_name(prefix))
    } else {
        None
    }
}

/// Cleans a name for use in filename
fn clean_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_junk_file() {
        assert!(is_junk_file(".DS_Store"));
        assert!(is_junk_file("Thumbs.db"));
        assert!(is_junk_file("__MACOSX/file.txt"));
        assert!(is_junk_file("readme.txt"));
        assert!(is_junk_file("LICENSE.txt"));

        assert!(!is_junk_file("document.pdf"));
        assert!(!is_junk_file("notes.txt"));
    }

    #[test]
    fn test_extract_stem() {
        assert_eq!(
            extract_stem("project/report.pdf"),
            Some("report".to_string())
        );
        assert_eq!(
            extract_stem("document.docx"),
            Some("document".to_string())
        );
        assert_eq!(extract_stem("a.txt"), None); // Too short
    }

    #[test]
    fn test_find_common_prefix() {
        let files = vec![
            "project_file1.txt".to_string(),
            "project_file2.txt".to_string(),
            "project_file3.txt".to_string(),
        ];

        let result = find_common_prefix(&files);
        assert_eq!(result, Some("project".to_string()));
    }

    #[test]
    fn test_find_common_prefix_no_common() {
        let files = vec![
            "abc.txt".to_string(),
            "xyz.txt".to_string(),
        ];

        let result = find_common_prefix(&files);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clean_name() {
        assert_eq!(clean_name("project-v1.2"), "project-v12");  // .2 becomes 2
        assert_eq!(clean_name("__test__"), "test");
        assert_eq!(clean_name("my_file_name"), "my_file_name");
    }
}
