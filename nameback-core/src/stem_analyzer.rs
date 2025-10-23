use std::path::Path;

/// Analyzes original filename stem to extract meaningful parts
/// Removes common prefixes and filters out date/numeric patterns
pub fn extract_meaningful_stem(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;

    // Remove common prefixes
    let cleaned = remove_common_prefixes(stem);

    // Split on common separators
    let parts: Vec<&str> = cleaned
        .split(['_', '-', '.', ' '])
        .filter(|s| !s.is_empty())
        .collect();

    // Filter out meaningless parts
    let meaningful: Vec<&str> = parts
        .iter()
        .filter(|p| is_meaningful_part(p))
        .copied()
        .collect();

    // Need at least 2 meaningful parts or 1 part with decent length
    if meaningful.len() >= 2 {
        Some(meaningful.join("_"))
    } else if meaningful.len() == 1 && meaningful[0].len() >= 5 {
        Some(meaningful[0].to_string())
    } else {
        None
    }
}

/// Removes common camera/screenshot prefixes
fn remove_common_prefixes(name: &str) -> String {
    let prefixes = [
        "IMG_", "DSC_", "SCAN_", "Screenshot_", "Capture_",
        "VID_", "Screen_Shot_", "Photo_", "Video_",
        "Document_", "Copy_of_", "Draft_", "New_",
        "Untitled_", "image_", "video_", "file_",
    ];

    let mut result = name.to_string();

    // Keep removing prefixes until none match (handles multiple prefixes)
    loop {
        let mut changed = false;
        for prefix in &prefixes {
            if let Some(stripped) = result.strip_prefix(prefix) {
                result = stripped.to_string();
                changed = true;
                break;
            }
            // Case-insensitive variant
            let lower_result = result.to_lowercase();
            let lower_prefix = prefix.to_lowercase();
            if lower_result.starts_with(&lower_prefix) {
                result = result[prefix.len()..].to_string();
                changed = true;
                break;
            }
        }
        if !changed {
            break;
        }
    }

    result
}

/// Checks if a part is meaningful (not just dates/numbers)
fn is_meaningful_part(part: &str) -> bool {
    // Too short
    if part.len() < 2 {
        return false;
    }

    // Pure numeric (likely a sequence number or date)
    if part.chars().all(|c| c.is_numeric()) {
        return false;
    }

    // Date patterns (YYYYMMDD, YYYY-MM-DD variants)
    if is_date_pattern(part) {
        return false;
    }

    // Timestamp patterns (HHMMSS, HH-MM-SS)
    if is_time_pattern(part) {
        return false;
    }

    // Version numbers (v1, v2, final, final2, rev3)
    if is_version_pattern(part) {
        return false;
    }

    // Must have some alphabetic characters
    part.chars().any(|c| c.is_alphabetic())
}

/// Detects date-like patterns
fn is_date_pattern(s: &str) -> bool {
    // Remove separators
    let cleaned: String = s.chars().filter(|c| c.is_alphanumeric()).collect();

    if !cleaned.chars().all(|c| c.is_numeric()) {
        return false;
    }

    // Common date lengths: YYYY (4), YYYYMM (6), YYYYMMDD (8)
    matches!(cleaned.len(), 4 | 6 | 8)
}

/// Detects time-like patterns (HHMMSS, etc.)
fn is_time_pattern(s: &str) -> bool {
    let cleaned: String = s.chars().filter(|c| c.is_alphanumeric()).collect();

    if !cleaned.chars().all(|c| c.is_numeric()) {
        return false;
    }

    // HHMMSS (6 digits) or HHMM (4 digits)
    if cleaned.len() == 6 || cleaned.len() == 4 {
        // Additional check: likely time if all digits are < 60
        if let Ok(num) = cleaned.parse::<u32>() {
            // Quick heuristic: times are usually < 240000 (23:59:59)
            return num < 240000;
        }
    }

    false
}

/// Detects version/revision patterns
fn is_version_pattern(s: &str) -> bool {
    let lower = s.to_lowercase();

    // v1, v2, version1, etc.
    if lower.starts_with('v') && lower[1..].chars().all(|c| c.is_numeric()) {
        return true;
    }

    // final, final2, final3
    if lower.starts_with("final") {
        return true;
    }

    // rev1, rev2, revision3
    if let Some(rest) = lower.strip_prefix("rev") {
        if rest.is_empty() || rest.chars().all(|c| c.is_numeric()) {
            return true;
        }
    }

    // copy, copy2, copy3
    if let Some(rest) = lower.strip_prefix("copy") {
        if rest.is_empty() || rest.chars().all(|c| c.is_numeric()) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_remove_common_prefixes() {
        assert_eq!(remove_common_prefixes("IMG_20231015"), "20231015");
        assert_eq!(remove_common_prefixes("Screenshot_MyProject"), "MyProject");
        assert_eq!(remove_common_prefixes("DSC_1234"), "1234");
        assert_eq!(remove_common_prefixes("Copy_of_Report"), "Report");
    }

    #[test]
    fn test_extract_meaningful_stem_with_good_parts() {
        let path = PathBuf::from("/test/IMG_Project_Proposal.jpg");
        assert_eq!(
            extract_meaningful_stem(&path),
            Some("Project_Proposal".to_string())
        );
    }

    #[test]
    fn test_extract_meaningful_stem_date_only() {
        let path = PathBuf::from("/test/IMG_20231015_143022.jpg");
        // All parts are dates/times, should return None
        assert_eq!(extract_meaningful_stem(&path), None);
    }

    #[test]
    fn test_extract_meaningful_stem_mixed() {
        let path = PathBuf::from("/test/Screenshot_2023_10_15_MyApp_Demo.png");
        // Should keep MyApp and Demo, filter out date parts
        assert_eq!(
            extract_meaningful_stem(&path),
            Some("MyApp_Demo".to_string())
        );
    }

    #[test]
    fn test_is_meaningful_part() {
        assert!(is_meaningful_part("Project"));
        assert!(is_meaningful_part("Report"));
        assert!(is_meaningful_part("v1a")); // has alpha

        assert!(!is_meaningful_part("123"));
        assert!(!is_meaningful_part("20231015"));
        assert!(!is_meaningful_part("v1"));
        assert!(!is_meaningful_part("final"));
        assert!(!is_meaningful_part("copy2"));
    }

    #[test]
    fn test_is_date_pattern() {
        assert!(is_date_pattern("20231015"));
        assert!(is_date_pattern("2023"));
        assert!(is_date_pattern("202310"));

        assert!(!is_date_pattern("Report"));
        assert!(!is_date_pattern("12")); // too short
        assert!(!is_date_pattern("123456789")); // too long
    }

    #[test]
    fn test_is_time_pattern() {
        assert!(is_time_pattern("143022")); // 14:30:22
        assert!(is_time_pattern("1430")); // 14:30

        assert!(!is_time_pattern("999999")); // > 24 hours
        assert!(!is_time_pattern("12")); // too short
    }

    #[test]
    fn test_is_version_pattern() {
        assert!(is_version_pattern("v1"));
        assert!(is_version_pattern("v12"));
        assert!(is_version_pattern("final"));
        assert!(is_version_pattern("final2"));
        assert!(is_version_pattern("rev"));
        assert!(is_version_pattern("rev3"));
        assert!(is_version_pattern("copy"));
        assert!(is_version_pattern("copy2"));

        assert!(!is_version_pattern("version")); // doesn't start with 'v' alone
        assert!(!is_version_pattern("Project"));
    }

    #[test]
    fn test_extract_with_multiple_prefixes() {
        let path = PathBuf::from("/test/Copy_of_Draft_Important_Document.docx");
        // Should remove both Copy_of_ and Draft_
        assert_eq!(
            extract_meaningful_stem(&path),
            Some("Important_Document".to_string())
        );
    }

    #[test]
    fn test_extract_single_long_meaningful_part() {
        let path = PathBuf::from("/test/IMG_ProjectProposal.jpg");
        // Single part but long enough
        assert_eq!(
            extract_meaningful_stem(&path),
            Some("ProjectProposal".to_string())
        );
    }

    #[test]
    fn test_extract_too_few_meaningful_parts() {
        let path = PathBuf::from("/test/IMG_12.jpg");
        // Only "12" which is numeric - should return None
        assert_eq!(extract_meaningful_stem(&path), None);
    }
}
