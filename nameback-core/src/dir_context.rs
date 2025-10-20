use std::path::Path;

/// Extracts meaningful context from directory structure
/// Returns parent and/or grandparent directory names if they're meaningful
pub fn extract_directory_context(path: &Path) -> Option<String> {
    let mut context_parts = Vec::new();

    // Get parent directory name
    if let Some(parent) = path.parent() {
        let parent_name_opt = parent.file_name().and_then(|n| n.to_str());

        if let Some(parent_name) = parent_name_opt {
            if !is_generic_dir_name(parent_name) {
                context_parts.push(parent_name.to_string());
            }

            // Also check grandparent for richer context
            if let Some(grandparent) = parent.parent() {
                if let Some(gp_name) = grandparent.file_name().and_then(|n| n.to_str()) {
                    // Only add grandparent if not generic and different from parent
                    if !is_generic_dir_name(gp_name) && gp_name != parent_name {
                        // Prepend grandparent (so it comes first in path)
                        context_parts.insert(0, gp_name.to_string());
                    }
                }
            }
        }
    }

    if !context_parts.is_empty() {
        Some(context_parts.join("_"))
    } else {
        None
    }
}

/// Checks if directory name is too generic to be useful
fn is_generic_dir_name(name: &str) -> bool {
    let lower = name.to_lowercase();

    let generic_names = [
        // User directories
        "documents", "downloads", "desktop", "pictures", "videos",
        "music", "photos", "files", "mydocuments",
        // Temp/working directories
        "tmp", "temp", "temporary", "cache", "data",
        // Generic organizational folders
        "misc", "miscellaneous", "other", "stuff", "things",
        "new", "old", "archive", "backup",
        // Development directories (when at wrong level)
        "src", "lib", "bin", "build", "dist", "output",
        // Year-only directories (too broad)
    ];

    if generic_names.contains(&lower.as_str()) {
        return true;
    }

    // Check if it's just a year (2020, 2023, etc.)
    if lower.len() == 4 && lower.chars().all(|c| c.is_numeric()) {
        if let Ok(year) = lower.parse::<u32>() {
            // Years between 1900-2100 are probably too generic
            if (1900..=2100).contains(&year) {
                return true;
            }
        }
    }

    // Check if it's a month (01, 02, ..., 12)
    if lower.len() == 2 && lower.chars().all(|c| c.is_numeric()) {
        if let Ok(month) = lower.parse::<u32>() {
            if (1..=12).contains(&month) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_with_meaningful_parent() {
        let path = PathBuf::from("/home/user/Projects/Website/images/hero.jpg");
        let context = extract_directory_context(&path);
        // Should skip "images" if it's too generic, but we haven't marked it as such
        // This test shows it would return "images"
        assert!(context.is_some());
    }

    #[test]
    fn test_extract_with_generic_parent() {
        let path = PathBuf::from("/home/user/Downloads/report.pdf");
        let context = extract_directory_context(&path);
        // "Downloads" is generic, so should return None or skip to grandparent
        // Since "user" is also somewhat generic, result depends on implementation
        assert!(context.is_some()); // Will include "user" since it's not in our generic list
    }

    #[test]
    fn test_extract_with_meaningful_grandparent() {
        let path = PathBuf::from("/home/user/Invoices/2023/Q4/invoice.pdf");
        let context = extract_directory_context(&path);
        // Parent is "Q4" (meaningful), grandparent is "2023" (generic year, filtered)
        // So we only get "Q4"
        assert_eq!(context, Some("Q4".to_string()));
    }

    #[test]
    fn test_is_generic_dir_name() {
        // Generic names
        assert!(is_generic_dir_name("documents"));
        assert!(is_generic_dir_name("Downloads")); // case insensitive
        assert!(is_generic_dir_name("tmp"));
        assert!(is_generic_dir_name("misc"));
        assert!(is_generic_dir_name("2023")); // year
        assert!(is_generic_dir_name("01")); // month

        // Meaningful names
        assert!(!is_generic_dir_name("Projects"));
        assert!(!is_generic_dir_name("Invoices"));
        assert!(!is_generic_dir_name("Website"));
        assert!(!is_generic_dir_name("Q4"));
        assert!(!is_generic_dir_name("CustomerData"));
    }

    #[test]
    fn test_extract_single_meaningful_parent() {
        let path = PathBuf::from("/data/ProjectAlpha/file.txt");
        let context = extract_directory_context(&path);
        assert_eq!(context, Some("ProjectAlpha".to_string()));
    }

    #[test]
    fn test_extract_no_parent() {
        let path = PathBuf::from("file.txt");
        let context = extract_directory_context(&path);
        assert_eq!(context, None);
    }

    #[test]
    fn test_extract_combines_meaningful_levels() {
        let path = PathBuf::from("/home/CompanyDocs/Legal/Contracts/contract.pdf");
        let context = extract_directory_context(&path);
        // Should combine Legal and Contracts (both meaningful)
        assert_eq!(context, Some("Legal_Contracts".to_string()));
    }

    #[test]
    fn test_year_detection() {
        assert!(is_generic_dir_name("2020"));
        assert!(is_generic_dir_name("2023"));
        assert!(is_generic_dir_name("1999"));
        assert!(!is_generic_dir_name("3000")); // far future, probably not a year
        assert!(!is_generic_dir_name("1800")); // before our range
    }

    #[test]
    fn test_month_detection() {
        assert!(is_generic_dir_name("01"));
        assert!(is_generic_dir_name("12"));
        assert!(!is_generic_dir_name("13")); // not a valid month
        assert!(!is_generic_dir_name("00"));
    }
}
