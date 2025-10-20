use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a detected file series
#[derive(Debug, Clone)]
pub struct FileSeries {
    pub base_name: String,
    pub files: Vec<(PathBuf, usize)>, // (path, sequence_number)
    pub pattern: SeriesPattern,
}

/// Pattern type for sequence numbering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeriesPattern {
    Underscore,  // name_001
    Parentheses, // name(001)
    Hyphen,      // name-001
    Space,       // name 001
}

impl SeriesPattern {
    /// Returns the regex pattern for this series type
    fn regex_pattern(&self) -> &str {
        match self {
            SeriesPattern::Underscore => r"^(.+?)_(\d+)$",
            SeriesPattern::Parentheses => r"^(.+?)\((\d+)\)$",
            SeriesPattern::Hyphen => r"^(.+?)-(\d+)$",
            SeriesPattern::Space => r"^(.+?)\s+(\d+)$",
        }
    }

    /// Formats a name with this pattern
    pub fn format(&self, base: &str, number: usize, width: usize) -> String {
        let num_str = format!("{:0width$}", number, width = width);
        match self {
            SeriesPattern::Underscore => format!("{}_{}", base, num_str),
            SeriesPattern::Parentheses => format!("{}({})", base, num_str),
            SeriesPattern::Hyphen => format!("{}-{}", base, num_str),
            SeriesPattern::Space => format!("{} {}", base, num_str),
        }
    }
}

/// Detects file series from a list of file paths
/// Returns series with 3+ members
pub fn detect_series(files: &[PathBuf]) -> Vec<FileSeries> {
    let mut series_map: HashMap<(String, SeriesPattern), Vec<(PathBuf, usize)>> = HashMap::new();

    // Try each pattern type
    for pattern_type in &[
        SeriesPattern::Underscore,
        SeriesPattern::Parentheses,
        SeriesPattern::Hyphen,
        SeriesPattern::Space,
    ] {
        let re = Regex::new(pattern_type.regex_pattern()).unwrap();

        for file_path in files {
            if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
                if let Some(captures) = re.captures(stem) {
                    let base = captures.get(1).unwrap().as_str().to_string();
                    let number: usize = captures.get(2).unwrap().as_str().parse().unwrap_or(0);

                    series_map
                        .entry((base.clone(), *pattern_type))
                        .or_insert_with(Vec::new)
                        .push((file_path.clone(), number));
                }
            }
        }
    }

    // Filter to series with 3+ members and convert to FileSeries
    series_map
        .into_iter()
        .filter(|(_, files)| files.len() >= 3)
        .map(|((base_name, pattern), mut files)| {
            // Sort by sequence number
            files.sort_by_key(|(_, num)| *num);
            FileSeries {
                base_name,
                files,
                pattern,
            }
        })
        .collect()
}

/// Applies series naming to a specific file within a series
pub fn apply_series_naming(
    series: &FileSeries,
    file_path: &Path,
    new_base_name: &str,
) -> Option<String> {
    // Find this file's sequence number in the series
    for (path, seq_num) in &series.files {
        if path == file_path {
            // Determine padding width (minimum 3, or number of digits in max sequence)
            let max_num = series.files.iter().map(|(_, n)| *n).max().unwrap_or(0);
            let width = format!("{}", max_num).len().max(3);

            // Get file extension
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let new_name = series.pattern.format(new_base_name, *seq_num, width);

            if extension.is_empty() {
                return Some(new_name);
            } else {
                return Some(format!("{}.{}", new_name, extension));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_series_underscore() {
        let files = vec![
            PathBuf::from("/test/IMG_001.jpg"),
            PathBuf::from("/test/IMG_002.jpg"),
            PathBuf::from("/test/IMG_003.jpg"),
        ];

        let series = detect_series(&files);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].base_name, "IMG");
        assert_eq!(series[0].pattern, SeriesPattern::Underscore);
        assert_eq!(series[0].files.len(), 3);
    }

    #[test]
    fn test_detect_series_parentheses() {
        let files = vec![
            PathBuf::from("/test/Screenshot(1).png"),
            PathBuf::from("/test/Screenshot(2).png"),
            PathBuf::from("/test/Screenshot(3).png"),
        ];

        let series = detect_series(&files);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].base_name, "Screenshot");
        assert_eq!(series[0].pattern, SeriesPattern::Parentheses);
    }

    #[test]
    fn test_detect_series_hyphen() {
        let files = vec![
            PathBuf::from("/test/report-1.pdf"),
            PathBuf::from("/test/report-2.pdf"),
            PathBuf::from("/test/report-3.pdf"),
        ];

        let series = detect_series(&files);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].base_name, "report");
        assert_eq!(series[0].pattern, SeriesPattern::Hyphen);
    }

    #[test]
    fn test_detect_series_requires_three_members() {
        let files = vec![
            PathBuf::from("/test/IMG_001.jpg"),
            PathBuf::from("/test/IMG_002.jpg"),
        ];

        let series = detect_series(&files);
        assert_eq!(series.len(), 0); // Need 3+ members
    }

    #[test]
    fn test_apply_series_naming() {
        let series = FileSeries {
            base_name: "IMG".to_string(),
            files: vec![
                (PathBuf::from("/test/IMG_001.jpg"), 1),
                (PathBuf::from("/test/IMG_002.jpg"), 2),
                (PathBuf::from("/test/IMG_003.jpg"), 3),
            ],
            pattern: SeriesPattern::Underscore,
        };

        let result = apply_series_naming(
            &series,
            &PathBuf::from("/test/IMG_002.jpg"),
            "vacation_photos",
        );

        assert_eq!(result, Some("vacation_photos_002.jpg".to_string()));
    }

    #[test]
    fn test_apply_series_naming_padding() {
        let series = FileSeries {
            base_name: "IMG".to_string(),
            files: vec![
                (PathBuf::from("/test/IMG_1.jpg"), 1),
                (PathBuf::from("/test/IMG_100.jpg"), 100),
            ],
            pattern: SeriesPattern::Underscore,
        };

        let result =
            apply_series_naming(&series, &PathBuf::from("/test/IMG_1.jpg"), "vacation");

        // Should pad to 3 digits (minimum width)
        assert_eq!(result, Some("vacation_001.jpg".to_string()));
    }

    #[test]
    fn test_pattern_format() {
        assert_eq!(
            SeriesPattern::Underscore.format("test", 5, 3),
            "test_005"
        );
        assert_eq!(
            SeriesPattern::Parentheses.format("test", 5, 3),
            "test(005)"
        );
        assert_eq!(SeriesPattern::Hyphen.format("test", 5, 3), "test-005");
        assert_eq!(SeriesPattern::Space.format("test", 5, 3), "test 005");
    }

    #[test]
    fn test_detect_multiple_series() {
        let files = vec![
            PathBuf::from("/test/IMG_001.jpg"),
            PathBuf::from("/test/IMG_002.jpg"),
            PathBuf::from("/test/IMG_003.jpg"),
            PathBuf::from("/test/VID_001.mp4"),
            PathBuf::from("/test/VID_002.mp4"),
            PathBuf::from("/test/VID_003.mp4"),
        ];

        let series = detect_series(&files);
        assert_eq!(series.len(), 2);

        let bases: Vec<&str> = series.iter().map(|s| s.base_name.as_str()).collect();
        assert!(bases.contains(&"IMG"));
        assert!(bases.contains(&"VID"));
    }
}
