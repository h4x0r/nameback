use std::collections::HashSet;

/// Represents a candidate name with its quality score
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NameCandidate {
    pub name: String,
    pub score: f32,
    pub source: NameSource,
}

/// Source of a candidate name
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum NameSource {
    Metadata,       // From EXIF/metadata fields
    TextExtract,    // Extracted from text content
    PdfText,        // Extracted from PDF text
    OcrImage,       // OCR from image
    OcrVideo,       // OCR from video frame
    DirectoryContext, // From directory structure
    FilenameAnalysis, // From analyzing original filename
    Fallback,       // Last resort (timestamp, etc.)
}

impl NameCandidate {
    /// Creates a new name candidate with calculated score
    pub fn new(name: String, source: NameSource) -> Self {
        let score = calculate_score(&name, source);
        Self { name, score, source }
    }

    /// Returns true if this candidate is high quality (score >= 5.0)
    #[allow(dead_code)]
    pub fn is_high_quality(&self) -> bool {
        self.score >= 5.0
    }

    /// Returns true if this candidate is acceptable (score >= 2.0)
    pub fn is_acceptable(&self) -> bool {
        self.score >= 2.0
    }
}

/// Calculates quality score for a candidate name
fn calculate_score(name: &str, source: NameSource) -> f32 {
    // Guard against empty strings
    if name.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;

    // 1. Length score (optimal 20-60 chars)
    let length_score = match name.len() {
        0..=10 => 0.2,
        11..=19 => 0.6,
        20..=60 => 1.0,
        61..=100 => 0.7,
        _ => 0.4,
    };
    score += length_score * 2.0;

    // 2. Source reliability
    let source_score = match source {
        NameSource::Metadata => 3.0,
        NameSource::TextExtract => 2.5,
        NameSource::PdfText => 2.0,
        NameSource::DirectoryContext => 1.8,
        NameSource::FilenameAnalysis => 1.5,
        NameSource::OcrImage => 1.5,
        NameSource::OcrVideo => 1.2,
        NameSource::Fallback => 0.5,
    };
    score += source_score;

    // 3. Word count bonus (encourages descriptive multi-word names)
    let word_count = name.split_whitespace().count();
    let word_bonus = (word_count.min(5) as f32) * 0.5;
    score += word_bonus;

    // 4. Character diversity (avoid "AAAA" or "1111")
    let unique_chars: HashSet<char> = name.chars().collect();
    let diversity = unique_chars.len() as f32 / name.len() as f32;
    score += diversity * 1.5;

    // 5. Apply penalties
    score = apply_penalties(name, score);

    score
}

/// Applies penalty multipliers for low-quality content
fn apply_penalties(name: &str, base_score: f32) -> f32 {
    let mut score = base_score;
    let lower = name.to_lowercase();

    // Date-only penalty
    if is_date_only_pattern(&lower) {
        score *= 0.3;
    }

    // Error indicator penalty
    let error_indicators = [
        "error", "exception", "warning", "failed", "cannot",
        "invalid", "traceback",
    ];
    if error_indicators.iter().any(|e| lower.contains(e)) {
        score *= 0.2;
    }

    // Technical ID penalty (UUIDs, hashes)
    if looks_like_technical_id(name) {
        score *= 0.3;
    }

    // Software installer pattern penalty
    if looks_like_installer(name) {
        score *= 0.2;
    }

    // Mostly numeric penalty
    let alpha_count = name.chars().filter(|c| c.is_alphabetic()).count();
    let numeric_ratio = name.chars().filter(|c| c.is_numeric()).count() as f32 / name.len() as f32;
    if alpha_count < 3 || numeric_ratio > 0.7 {
        score *= 0.5;
    }

    score
}

/// Checks if name is just a date pattern (public for use in extractor)
pub fn is_date_only_pattern(s: &str) -> bool {
    let cleaned: String = s.chars().filter(|c| c.is_alphanumeric()).collect();

    if !cleaned.chars().all(|c| c.is_numeric()) {
        return false;
    }

    matches!(cleaned.len(), 4 | 6 | 8)
}

/// Checks if name looks like a technical ID (UUID, hash, serial number)
fn looks_like_technical_id(s: &str) -> bool {
    // UUID pattern (8-4-4-4-12)
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 5 {
        let lens: Vec<usize> = parts.iter().map(|p| p.len()).collect();
        if lens == vec![8, 4, 4, 4, 12] {
            return true;
        }
    }

    // Hash-like (long hex string)
    if s.len() >= 32 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    false
}

/// Helper to detect if string contains a recent year (20XX)
fn contains_recent_year(s: &str) -> bool {
    // Match 4-digit years from 2010-2030 (reasonable installer date range)
    for year in 2010..=2030 {
        if s.contains(&year.to_string()) {
            return true;
        }
    }
    false
}

/// Checks if name looks like a software installer/package filename
fn looks_like_installer(name: &str) -> bool {
    let lower = name.to_lowercase();

    // Count indicators of installer patterns
    let mut indicator_count = 0;

    // Platform identifiers
    let platforms = [
        "windows", "win32", "win64", "macos", "osx", "darwin",
        "linux", "ubuntu", "debian", "x86", "x64", "amd64", "arm64",
    ];
    if platforms.iter().any(|p| lower.contains(p)) {
        indicator_count += 1;
    }

    // Version number patterns (decimal versions like 17.4, 2.1.3)
    if lower.chars().filter(|c| *c == '.').count() >= 1 {
        // Check if there are numeric sequences around dots
        let parts: Vec<&str> = lower.split(&[' ', '_', '-'][..]).collect();
        for part in parts {
            if is_decimal_version_pattern(part) {
                indicator_count += 1;
                break;
            }
        }
    }

    // Common software vendors
    let vendors = ["adobe", "microsoft", "google", "apple", "oracle"];
    if vendors.iter().any(|v| lower.contains(v)) {
        indicator_count += 1;
    }

    // Date patterns with recent years
    if contains_recent_year(&lower) {
        indicator_count += 1;
    }

    // Installer-specific keywords
    let installer_keywords = ["setup", "install", "installer", "package", "release"];
    if installer_keywords.iter().any(|k| lower.contains(k)) {
        indicator_count += 1;
    }

    // If we have 3 or more indicators, this is likely an installer filename
    indicator_count >= 3
}

/// Helper to detect decimal version pattern (1.2, 17.4, etc.)
fn is_decimal_version_pattern(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return false;
    }
    parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_numeric()))
}

/// Selects the best candidate from a list based on scores
pub fn select_best_candidate(candidates: Vec<NameCandidate>) -> Option<NameCandidate> {
    if candidates.is_empty() {
        return None;
    }

    // Find candidate with highest score
    let best = candidates
        .into_iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())?;

    // Only return if acceptable quality
    if best.is_acceptable() {
        Some(best)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_high_quality_metadata() {
        let candidate = NameCandidate::new(
            "Quarterly Sales Report Q3 2023".to_string(),
            NameSource::Metadata,
        );
        assert!(candidate.score > 8.0, "Expected score > 8.0, got {}", candidate.score);
        assert!(candidate.is_high_quality());
    }

    #[test]
    fn test_score_poor_quality_ocr() {
        let candidate = NameCandidate::new(
            "IMG_20231015".to_string(),
            NameSource::OcrImage,
        );
        assert!(candidate.score < 5.0);
        assert!(!candidate.is_high_quality());
    }

    #[test]
    fn test_score_date_only() {
        let candidate = NameCandidate::new(
            "20231015".to_string(),
            NameSource::TextExtract,
        );
        assert!(candidate.score < 2.0, "Date-only should score < 2.0, got {}", candidate.score);
        assert!(!candidate.is_acceptable());
    }

    #[test]
    fn test_score_error_message() {
        let candidate = NameCandidate::new(
            "ERROR: Cannot read file".to_string(),
            NameSource::OcrImage,
        );
        // Error penalty is 0.2x multiplier, so score should be very low
        assert!(candidate.score < 2.0, "Error message should score < 2.0, got {}", candidate.score);
        assert!(!candidate.is_acceptable());
    }

    #[test]
    fn test_score_uuid() {
        let candidate = NameCandidate::new(
            "a3d5e7f9-1234-5678-90ab-cdef12345678".to_string(),
            NameSource::FilenameAnalysis,
        );
        assert!(candidate.score < 2.0);
    }

    #[test]
    fn test_select_best_from_multiple() {
        let candidates = vec![
            NameCandidate::new("IMG_123".to_string(), NameSource::OcrImage),
            NameCandidate::new("Project Proposal Draft".to_string(), NameSource::Metadata),
            NameCandidate::new("20231015".to_string(), NameSource::FilenameAnalysis),
        ];

        let best = select_best_candidate(candidates).unwrap();
        assert_eq!(best.name, "Project Proposal Draft");
        assert!(best.score > 7.0, "Expected score > 7.0, got {}", best.score);
        assert!(best.is_high_quality());
    }

    #[test]
    fn test_select_best_rejects_all_low_quality() {
        let candidates = vec![
            NameCandidate::new("123".to_string(), NameSource::Fallback),
            NameCandidate::new("ab".to_string(), NameSource::OcrImage),
        ];

        let best = select_best_candidate(candidates);
        assert!(best.is_none(), "Should reject all low quality candidates");
    }

    #[test]
    fn test_length_scoring() {
        // Too short
        let short = NameCandidate::new("ab".to_string(), NameSource::Metadata);

        // Optimal
        let optimal = NameCandidate::new("Project Budget Analysis Report".to_string(), NameSource::Metadata);

        // Too long
        let long = NameCandidate::new("a".repeat(150), NameSource::Metadata);

        assert!(optimal.score > short.score);
        assert!(optimal.score > long.score);
    }

    #[test]
    fn test_word_count_bonus() {
        let single = NameCandidate::new("Report".to_string(), NameSource::Metadata);
        let multi = NameCandidate::new("Quarterly Sales Report".to_string(), NameSource::Metadata);

        assert!(multi.score > single.score);
    }

    #[test]
    fn test_diversity_bonus() {
        let repetitive = NameCandidate::new("aaaaaaa".to_string(), NameSource::OcrImage);
        let diverse = NameCandidate::new("Meeting".to_string(), NameSource::OcrImage);

        assert!(diverse.score > repetitive.score);
    }

    #[test]
    fn test_installer_pattern_adobe() {
        let candidate = NameCandidate::new(
            "Adobe_InDesign_17.4_(Windows)_2022-12-08".to_string(),
            NameSource::FilenameAnalysis,
        );
        // Should have heavy penalty due to installer pattern
        assert!(candidate.score < 2.0, "Installer pattern should score < 2.0, got {}", candidate.score);
        assert!(!candidate.is_acceptable());
    }

    #[test]
    fn test_installer_pattern_generic() {
        let candidate = NameCandidate::new(
            "MyApp_3.2_Linux_x86_64_Setup".to_string(),
            NameSource::FilenameAnalysis,
        );
        // Should have heavy penalty
        assert!(candidate.score < 2.0);
        assert!(!candidate.is_acceptable());
    }

    #[test]
    fn test_not_installer_regular_doc() {
        let candidate = NameCandidate::new(
            "Project Proposal Draft".to_string(),
            NameSource::Metadata,
        );
        // Should NOT be penalized as installer
        assert!(candidate.score > 5.0);
        assert!(candidate.is_high_quality());
    }

    #[test]
    fn test_is_decimal_version_pattern() {
        assert!(is_decimal_version_pattern("17.4"));
        assert!(is_decimal_version_pattern("1.2.3"));
        assert!(is_decimal_version_pattern("10.15.7"));

        assert!(!is_decimal_version_pattern("v1.2"));
        assert!(!is_decimal_version_pattern("Project"));
        assert!(!is_decimal_version_pattern("1"));
    }
}
