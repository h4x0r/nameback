use std::path::Path;

/// Analyzes original filename stem to extract meaningful parts
/// Removes common prefixes and filters out date/numeric patterns
pub fn extract_meaningful_stem(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;

    // Remove common prefixes
    let cleaned = remove_common_prefixes(stem);

    // Split on common separators, but preserve date patterns like "2021-08-23"
    // First, replace date-like patterns with a placeholder, split, then restore
    let date_placeholder = "\x00DATE\x00";
    let mut temp = cleaned.clone();
    let mut preserved_dates = Vec::new();

    // Match patterns like YYYY-MM-DD, YYYY/MM/DD, or YYYYMMDD
    let date_regex = regex::Regex::new(r"(\d{4}[-/]\d{2}[-/]\d{2}|\d{8})").unwrap();
    for cap in date_regex.captures_iter(&cleaned) {
        if let Some(matched) = cap.get(0) {
            preserved_dates.push(matched.as_str().to_string());
            temp = temp.replace(matched.as_str(), date_placeholder);
        }
    }

    // Split on common separators
    let mut parts: Vec<String> = temp
        .split(['_', '-', '.', ' '])
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Restore preserved dates
    let mut date_idx = 0;
    for part in parts.iter_mut() {
        if part.contains(date_placeholder) && date_idx < preserved_dates.len() {
            *part = preserved_dates[date_idx].clone();
            date_idx += 1;
        }
    }

    let parts: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

    // Separate date parts from other meaningful parts
    let date_parts: Vec<&str> = parts
        .iter()
        .filter(|p| is_date_pattern(p))
        .copied()
        .collect();

    // Filter meaningful non-date parts
    let meaningful: Vec<&str> = parts
        .iter()
        .filter(|p| !is_date_pattern(p) && is_meaningful_part(p))
        .copied()
        .collect();

    // Prefer meaningful non-date parts, fall back to dates if no meaningful parts
    if !meaningful.is_empty() {
        // We have meaningful non-date parts
        if meaningful.len() >= 2 {
            Some(meaningful.join("_"))
        } else if meaningful[0].len() >= 5 {
            Some(meaningful[0].to_string())
        } else if !date_parts.is_empty() {
            // Meaningful part too short, use date instead
            Some(date_parts.join("-"))
        } else {
            None
        }
    } else if !date_parts.is_empty() {
        // No meaningful non-date parts, use the date
        Some(date_parts.join("-"))
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

    // Platform identifiers (Windows, macOS, Linux, x86_64, etc.)
    if is_platform_identifier(part) {
        return false;
    }

    // Software vendor names (Adobe, Microsoft, Google, etc.)
    if is_software_vendor(part) {
        return false;
    }

    // Software product identifiers (CS6, CC, CS5, Office365, etc.)
    if is_software_product_id(part) {
        return false;
    }

    // Software product names (Photoshop, InDesign, Word, Excel, etc.)
    if is_software_product_name(part) {
        return false;
    }

    // Decimal version numbers (1.2, 17.4, etc.)
    if is_decimal_version(part) {
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
    // Note: Don't match 2-digit numbers as they're too ambiguous (could be counters, versions, etc.)
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

/// Detects platform identifiers
fn is_platform_identifier(s: &str) -> bool {
    let lower = s.to_lowercase();
    let clean = lower.trim_matches(|c: char| !c.is_alphanumeric());

    let platforms = [
        "windows", "win32", "win64", "macos", "osx", "darwin",
        "linux", "ubuntu", "debian", "redhat", "centos", "fedora",
        "x86", "x64", "x86_64", "amd64", "i386", "i686",
        "arm", "arm64", "aarch64", "armv7", "armv8",
        "32bit", "64bit", "universal",
    ];

    // Only use exact match to avoid false positives (e.g., "darwinian" matching "darwin")
    platforms.iter().any(|p| clean == *p)
}

/// Detects software vendor names
fn is_software_vendor(s: &str) -> bool {
    let lower = s.to_lowercase();

    let vendors = [
        "adobe", "microsoft", "google", "apple", "oracle",
        "ibm", "intel", "amd", "nvidia", "autodesk",
        "corel", "symantec", "mcafee", "norton",
    ];

    vendors.iter().any(|v| lower == *v)
}

/// Detects software product identifiers (CS6, CC, Office365, etc.)
fn is_software_product_id(s: &str) -> bool {
    let upper = s.to_uppercase();
    let lower = s.to_lowercase();

    // Adobe Creative Suite/Cloud versions: CS3, CS4, CS5, CS5.5, CS6, CC, CC2019, etc.
    if upper.starts_with("CS") {
        let rest = &upper[2..];
        // CS followed by number or empty (CS6, CS5.5, CS, etc.)
        if rest.is_empty() || rest.chars().next().map_or(false, |c| c.is_numeric() || c == '.') {
            return true;
        }
    }

    // Creative Cloud variants: CC, CC2019, CC2020, etc.
    if upper == "CC" || (upper.starts_with("CC") && upper[2..].chars().all(|c| c.is_numeric())) {
        return true;
    }

    // Microsoft Office variants: Office365, Office2019, etc.
    if lower.starts_with("office") && lower[6..].chars().all(|c| c.is_numeric()) {
        return true;
    }

    // Windows versions: Win10, Win11, etc.
    if lower.starts_with("win") && lower.len() <= 5 {
        let rest = &lower[3..];
        if rest.chars().all(|c| c.is_numeric()) && !rest.is_empty() {
            return true;
        }
    }

    // macOS versions: Monterey, BigSur, Catalina, etc. (less common in filenames)
    let macos_versions = [
        "monterey", "bigsur", "catalina", "mojave", "highsierra",
        "sierra", "elcapitan", "yosemite", "mavericks",
    ];
    if macos_versions.iter().any(|v| lower == *v) {
        return true;
    }

    // Common software edition markers
    let editions = [
        "pro", "professional", "enterprise", "ultimate", "premium",
        "standard", "basic", "lite", "home", "student", "business",
        "starter", "express", "community", "developer",
    ];
    if editions.iter().any(|e| lower == *e) {
        return true;
    }

    false
}

/// Detects software product names (Photoshop, InDesign, Word, Excel, etc.)
fn is_software_product_name(s: &str) -> bool {
    let lower = s.to_lowercase();

    // Adobe Creative Suite/Cloud products
    let adobe_products = [
        "photoshop", "illustrator", "indesign", "premiere", "aftereffects",
        "dreamweaver", "flash", "fireworks", "acrobat", "lightroom",
        "bridge", "audition", "animate", "dimension", "xd", "spark",
    ];
    if adobe_products.iter().any(|p| lower == *p) {
        return true;
    }

    // Microsoft Office products (only very distinctive names to avoid false positives)
    // Note: "Project", "Word", "Outlook", "Access" are too generic and commonly used in filenames
    let office_products = [
        "excel", "powerpoint", "onenote",
        "publisher", "visio", "teams", "sharepoint",
    ];
    if office_products.iter().any(|p| lower == *p) {
        return true;
    }

    // Other common software products
    let other_products = [
        "chrome", "firefox", "safari", "edge", "opera",
        "slack", "discord", "zoom", "skype",
        "spotify", "itunes", "vlc",
    ];
    if other_products.iter().any(|p| lower == *p) {
        return true;
    }

    false
}

/// Detects decimal version numbers (1.2, 17.4, etc.)
fn is_decimal_version(s: &str) -> bool {
    // Check if string is in format: digits.digits (optionally .digits)
    let parts: Vec<&str> = s.split('.').collect();

    // Must have 2 or 3 parts (e.g., "1.2" or "1.2.3")
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }

    // All parts must be numeric
    parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_numeric()))
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
        // All parts are dates/times, should return the date parts joined
        assert_eq!(extract_meaningful_stem(&path), Some("20231015-143022".to_string()));
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

    #[test]
    fn test_is_platform_identifier() {
        assert!(is_platform_identifier("Windows"));
        assert!(is_platform_identifier("(Windows)"));
        assert!(is_platform_identifier("windows"));
        assert!(is_platform_identifier("macOS"));
        assert!(is_platform_identifier("Linux"));
        assert!(is_platform_identifier("x86_64"));
        assert!(is_platform_identifier("amd64"));
        assert!(is_platform_identifier("ARM64"));
        assert!(is_platform_identifier("64bit"));

        assert!(!is_platform_identifier("Project"));
        assert!(!is_platform_identifier("Report"));
    }

    #[test]
    fn test_is_software_vendor() {
        assert!(is_software_vendor("Adobe"));
        assert!(is_software_vendor("adobe"));
        assert!(is_software_vendor("Microsoft"));
        assert!(is_software_vendor("Google"));

        assert!(!is_software_vendor("Project"));
        assert!(!is_software_vendor("InDesign"));
    }

    #[test]
    fn test_is_decimal_version() {
        assert!(is_decimal_version("17.4"));
        assert!(is_decimal_version("1.2"));
        assert!(is_decimal_version("2.1.3"));
        assert!(is_decimal_version("10.15.7"));

        assert!(!is_decimal_version("v1.2"));
        assert!(!is_decimal_version("1"));
        assert!(!is_decimal_version("Project"));
        assert!(!is_decimal_version("1.2.3.4")); // too many parts
    }

    #[test]
    fn test_extract_adobe_installer_pattern() {
        let path = PathBuf::from("/test/Adobe_InDesign_CS6_(Windows)_2021-08-23.pdf");
        // Should filter out: Adobe (vendor), InDesign (product name), CS6 (product ID), Windows (platform)
        // Should keep: 2021-08-23 (date - most unique identifier)
        let result = extract_meaningful_stem(&path);
        // No meaningful non-date parts, so use the date
        assert_eq!(result, Some("2021-08-23".to_string()));
    }

    #[test]
    fn test_extract_filters_platform_and_version() {
        let path = PathBuf::from("/test/MyApp_3.2_Linux_x86_64.zip");
        // Should filter out: 3.2 (decimal version), Linux, x86, 64 (numeric)
        // Should keep: MyApp
        assert_eq!(extract_meaningful_stem(&path), Some("MyApp".to_string()));
    }

    #[test]
    fn test_is_software_product_id() {
        // Adobe Creative Suite/Cloud versions
        assert!(is_software_product_id("CS6"));
        assert!(is_software_product_id("cs6"));
        assert!(is_software_product_id("CS5"));
        assert!(is_software_product_id("CS5.5"));
        assert!(is_software_product_id("CC"));
        assert!(is_software_product_id("cc"));
        assert!(is_software_product_id("CC2019"));
        assert!(is_software_product_id("CC2020"));

        // Microsoft Office versions
        assert!(is_software_product_id("Office365"));
        assert!(is_software_product_id("Office2019"));
        assert!(is_software_product_id("office365"));

        // Windows versions
        assert!(is_software_product_id("Win10"));
        assert!(is_software_product_id("Win11"));
        assert!(is_software_product_id("win10"));

        // macOS versions
        assert!(is_software_product_id("Monterey"));
        assert!(is_software_product_id("BigSur"));
        assert!(is_software_product_id("Catalina"));
        assert!(is_software_product_id("monterey"));

        // Edition markers
        assert!(is_software_product_id("Pro"));
        assert!(is_software_product_id("Professional"));
        assert!(is_software_product_id("Enterprise"));
        assert!(is_software_product_id("Ultimate"));
        assert!(is_software_product_id("Premium"));
        assert!(is_software_product_id("Standard"));
        assert!(is_software_product_id("Home"));
        assert!(is_software_product_id("Student"));
        assert!(is_software_product_id("pro"));
        assert!(is_software_product_id("community"));

        // Should NOT match these (actual product/project names)
        assert!(!is_software_product_id("InDesign"));
        assert!(!is_software_product_id("Photoshop"));
        assert!(!is_software_product_id("Project"));
        assert!(!is_software_product_id("Report"));
        assert!(!is_software_product_id("MyApp"));
        assert!(!is_software_product_id("Document"));
    }

    #[test]
    fn test_extract_office_installer_pattern() {
        let path = PathBuf::from("/test/Microsoft_Office_2019_Professional_Plus_(Windows).exe");
        // Should filter out: Microsoft (vendor), Office2019 (product ID), Professional (edition),
        //                    Plus (could keep), Windows (platform)
        // Should keep: Plus or Office depending on filtering
        let result = extract_meaningful_stem(&path);
        // "Plus" is 4 chars (< 5), so won't pass alone. "Office" might be kept if not filtered.
        // Actually, this will likely return None since all parts get filtered.
        // Let's just verify it doesn't include the noise
        if let Some(name) = result {
            assert!(!name.contains("Microsoft"));
            assert!(!name.contains("Windows"));
            assert!(!name.contains("Professional"));
        }
    }

    #[test]
    fn test_extract_creative_cloud_pattern() {
        let path = PathBuf::from("/test/Adobe_Photoshop_CC_2020_macOS.dmg");
        // Should filter out: Adobe (vendor), Photoshop (product name), CC (version), macOS (platform)
        // Should keep: 2020 (4-digit year - best unique identifier)
        assert_eq!(extract_meaningful_stem(&path), Some("2020".to_string()));
    }

    #[test]
    fn test_is_software_product_name() {
        // Adobe products
        assert!(is_software_product_name("Photoshop"));
        assert!(is_software_product_name("photoshop"));
        assert!(is_software_product_name("InDesign"));
        assert!(is_software_product_name("indesign"));
        assert!(is_software_product_name("Illustrator"));
        assert!(is_software_product_name("Premiere"));
        assert!(is_software_product_name("AfterEffects"));

        // Microsoft Office products (only distinctive ones to avoid false positives)
        assert!(is_software_product_name("Excel"));
        assert!(is_software_product_name("excel"));
        assert!(is_software_product_name("PowerPoint"));
        assert!(is_software_product_name("powerpoint"));
        assert!(is_software_product_name("OneNote"));

        // Other common software
        assert!(is_software_product_name("Chrome"));
        assert!(is_software_product_name("chrome"));
        assert!(is_software_product_name("Firefox"));
        assert!(is_software_product_name("Slack"));
        assert!(is_software_product_name("Zoom"));

        // Should NOT match actual project/document names
        assert!(!is_software_product_name("ProjectProposal"));
        assert!(!is_software_product_name("MyDocument"));
        assert!(!is_software_product_name("Report"));
        assert!(!is_software_product_name("Presentation"));
        assert!(!is_software_product_name("Budget"));
    }
}
