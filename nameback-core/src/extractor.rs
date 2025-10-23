use anyhow::{Context, Result};
use log::debug;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::detector::FileCategory;
use crate::image_ocr;
use crate::pdf_content;
use crate::text_content;
use crate::video_ocr;

/// Represents metadata extracted from a file
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub date_time_original: Option<String>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<String>,
    pub gps_location: Option<crate::location_timestamp::LocationData>,
    pub geocode_enabled: Option<bool>,
    pub include_location: bool,
    pub include_timestamp: bool,
}

impl FileMetadata {
    /// Extracts the best candidate name from metadata based on file category
    /// Now uses intelligent scoring to select from multiple sources
    pub fn extract_name(&self, category: &FileCategory, path: &Path) -> Option<String> {
        use crate::scorer::{NameCandidate, NameSource};

        let mut candidates = Vec::new();

        // Collect candidates from metadata fields based on category
        match category {
            FileCategory::Image => {
                if let Some(title) = &self.title {
                    candidates.push(NameCandidate::new(title.clone(), NameSource::Metadata));
                }
                if let Some(desc) = &self.description {
                    candidates.push(NameCandidate::new(desc.clone(), NameSource::Metadata));
                }
                if let Some(date) = &self.date_time_original {
                    candidates.push(NameCandidate::new(date.clone(), NameSource::Metadata));
                }
            }
            FileCategory::Document => {
                if let Some(title) = &self.title {
                    candidates.push(NameCandidate::new(title.clone(), NameSource::Metadata));
                }
                if let Some(subject) = &self.subject {
                    candidates.push(NameCandidate::new(subject.clone(), NameSource::Metadata));
                }
                if let Some(author) = &self.author {
                    candidates.push(NameCandidate::new(author.clone(), NameSource::Metadata));
                }
            }
            FileCategory::Audio => {
                if let Some(title) = &self.title {
                    candidates.push(NameCandidate::new(title.clone(), NameSource::Metadata));
                }
                if let Some(artist) = &self.artist {
                    candidates.push(NameCandidate::new(artist.clone(), NameSource::Metadata));
                }
                if let Some(album) = &self.album {
                    candidates.push(NameCandidate::new(album.clone(), NameSource::Metadata));
                }
            }
            FileCategory::Video => {
                if let Some(title) = &self.title {
                    candidates.push(NameCandidate::new(title.clone(), NameSource::Metadata));
                }
                if let Some(date) = &self.creation_date {
                    candidates.push(NameCandidate::new(date.clone(), NameSource::Metadata));
                }
            }
            FileCategory::Email => {
                // Email files handled by format handler
                if let Ok(email_meta) = crate::format_handlers::email::extract_email_metadata(path) {
                    if let Some(name) = crate::format_handlers::email::format_email_filename(&email_meta) {
                        candidates.push(NameCandidate::new(name, NameSource::Metadata));
                    }
                }
            }
            FileCategory::Web => {
                // Web files handled by format handler
                if let Ok(Some(title)) = crate::format_handlers::web::extract_html_title(path) {
                    candidates.push(NameCandidate::new(title, NameSource::Metadata));
                }
            }
            FileCategory::Archive => {
                // Archive files handled by format handler
                if let Ok(Some(name)) = crate::format_handlers::archive::extract_archive_info(path) {
                    candidates.push(NameCandidate::new(name, NameSource::Metadata));
                }
            }
            FileCategory::SourceCode => {
                // Source code files handled by docstring extractor
                if let Ok(Some(docstring)) = crate::code_docstring::extract_docstring(path) {
                    candidates.push(NameCandidate::new(docstring, NameSource::Metadata));
                }
            }
            FileCategory::Unknown => {}
        }

        // Try intelligent filename stem analysis
        if let Some(stem) = crate::stem_analyzer::extract_meaningful_stem(path) {
            candidates.push(NameCandidate::new(stem, NameSource::FilenameAnalysis));
        }

        // Try directory context
        if let Some(context) = crate::dir_context::extract_directory_context(path) {
            candidates.push(NameCandidate::new(context, NameSource::DirectoryContext));
        }

        // Use scorer to select best candidate
        crate::scorer::select_best_candidate(candidates).map(|c| c.name)
    }
}

/// Extracts metadata from a file using exiftool
pub fn extract_metadata(path: &Path, config: &crate::RenameConfig) -> Result<FileMetadata> {
    let output = Command::new("exiftool")
        .arg("-json")
        .arg(path)
        .output()
        .context("Failed to execute `exiftool` command. Is exiftool installed?")?;

    if !output.status.success() {
        anyhow::bail!("exiftool command failed with status: {}", output.status);
    }

    let json_str = String::from_utf8_lossy(&output.stdout);

    // exiftool returns an array with one object
    #[derive(Deserialize)]
    struct ExiftoolOutput {
        #[serde(rename = "Title")]
        title: Option<String>,
        #[serde(rename = "Artist")]
        artist: Option<String>,
        #[serde(rename = "Album")]
        album: Option<String>,
        #[serde(rename = "DateTimeOriginal")]
        date_time_original: Option<String>,
        #[serde(rename = "Description")]
        description: Option<String>,
        #[serde(rename = "Subject")]
        subject: Option<String>,
        #[serde(rename = "Author")]
        author: Option<String>,
        #[serde(rename = "Creator")]
        creator: Option<String>,
        #[serde(rename = "LastModifiedBy")]
        last_modified_by: Option<String>,
        #[serde(rename = "CreationDate")]
        creation_date: Option<String>,
        #[serde(rename = "CreateDate")]
        create_date: Option<String>,
        #[serde(rename = "GPSLatitude")]
        gps_latitude: Option<String>,
        #[serde(rename = "GPSLatitudeRef")]
        gps_latitude_ref: Option<String>,
        #[serde(rename = "GPSLongitude")]
        gps_longitude: Option<String>,
        #[serde(rename = "GPSLongitudeRef")]
        gps_longitude_ref: Option<String>,
    }

    let parsed: Vec<ExiftoolOutput> =
        serde_json::from_str(&json_str).context("Failed to parse exiftool JSON output")?;

    let exif_data = parsed
        .into_iter()
        .next()
        .context("No metadata found in exiftool output")?;

    let author = exif_data
        .author
        .or(exif_data.creator)
        .or(exif_data.last_modified_by);

    // Filter out unhelpful author names (like scanner/printer names)
    let filtered_author = if is_useful_metadata(&author) {
        author
    } else {
        None
    };

    // Extract GPS location if available
    let gps_location = crate::location_timestamp::extract_gps_from_metadata(
        exif_data.gps_latitude.as_deref(),
        exif_data.gps_latitude_ref.as_deref(),
        exif_data.gps_longitude.as_deref(),
        exif_data.gps_longitude_ref.as_deref(),
    );

    let mut metadata = FileMetadata {
        title: exif_data.title,
        artist: exif_data.artist,
        album: exif_data.album,
        date_time_original: exif_data.date_time_original,
        description: exif_data.description,
        subject: exif_data.subject,
        author: filtered_author,
        creation_date: exif_data.creation_date.or(exif_data.create_date),
        gps_location,
        geocode_enabled: Some(config.geocode),
        include_location: config.include_location,
        include_timestamp: config.include_timestamp,
    };

    // For PDFs without useful metadata, try extracting text content
    if is_pdf(path)
        && !is_useful_metadata(&metadata.title)
        && !is_useful_metadata(&metadata.subject)
    {
        debug!("PDF has no useful metadata, attempting content extraction");
        if let Ok(Some(content)) = pdf_content::extract_pdf_content(path) {
            debug!("Extracted PDF content: {}", content);
            metadata.title = Some(content);
        }
    }

    // For plain text files without useful metadata, try extracting content
    if is_text_file(path) && !has_any_useful_metadata(&metadata) {
        debug!("Text file has no useful metadata, attempting content extraction");
        if let Ok(Some(content)) = text_content::extract_text_content(path) {
            debug!("Extracted text content: {}", content);
            metadata.title = Some(content);
        }
    }

    // For images without useful metadata, try OCR
    if is_image(path) && !has_any_useful_metadata(&metadata) {
        debug!("Image has no useful metadata, attempting OCR");
        if let Ok(Some(text)) = image_ocr::extract_image_text(path) {
            debug!("Extracted image text: {}", text);
            metadata.title = Some(text);
        }
    }

    // For videos without useful metadata, try extracting and OCR'ing a frame
    if is_video(path)
        && !is_useful_metadata(&metadata.title)
        && !is_useful_metadata(&metadata.creation_date)
    {
        debug!("Video has no useful metadata, attempting frame extraction and OCR");
        let video_text = if config.multiframe_video {
            debug!("Using multi-frame video analysis (default)");
            video_ocr::extract_video_text_multiframe(path)
        } else {
            debug!("Using single-frame video analysis (--fast-video)");
            video_ocr::extract_video_text(path)
        };

        if let Ok(Some(text)) = video_text {
            debug!("Extracted video text: {}", text);
            metadata.title = Some(text);
        }
    }

    Ok(metadata)
}

/// Checks if a file is a PDF based on extension
fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

/// Checks if a file is an image based on extension
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "heic" | "heif"
            )
        })
        .unwrap_or(false)
}

/// Checks if a file is a video based on extension
fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "wmv" | "m4v"
            )
        })
        .unwrap_or(false)
}

/// Checks if a file is a plain text file based on extension
fn is_text_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "txt" | "text" | "md" | "markdown" | "csv" | "json" | "yaml" | "yml"
            )
        })
        .unwrap_or(false)
}

/// Checks if metadata string is useful (not empty, not just scanner/printer names)
fn is_useful_metadata(value: &Option<String>) -> bool {
    if let Some(v) = value {
        let lower = v.to_lowercase();

        // Basic length check
        if lower.is_empty() || lower.len() < 3 {
            return false;
        }

        // Error messages and warnings
        let error_indicators = [
            "error", "exception", "warning", "failed", "cannot",
            "invalid", "undefined", "null", "errno", "traceback",
            "fatal", "critical",
        ];
        if error_indicators.iter().any(|e| lower.contains(e)) {
            return false;
        }

        // Scanner and device names (expanded list)
        let device_indicators = [
            "canon", "printer", "scanner", "ipr", "epson", "hp",
            "brother", "xerox", "kyocera", "ricoh", "lexmark",
            "dell", "fujitsu",
        ];
        if device_indicators.iter().any(|d| lower.contains(d)) {
            return false;
        }

        // Generic placeholder text
        let generic_indicators = [
            "untitled", "new document", "document1", "image1",
            "noname", "unnamed", "temp", "test", "sample",
            "copy of", "draft",
        ];
        if generic_indicators.iter().any(|g| lower.contains(g)) {
            return false;
        }

        // Date-only strings (YYYY-MM-DD, YYYYMMDD, MM/DD/YYYY patterns)
        if crate::scorer::is_date_only_pattern(&lower) {
            return false;
        }

        // Mostly punctuation or symbols (less than 1/3 alphanumeric)
        let alpha_count = v.chars().filter(|c| c.is_alphanumeric()).count();
        if alpha_count < v.len() / 3 {
            return false;
        }

        // Excessive character repetition
        if has_excessive_repetition(&lower) {
            return false;
        }

        true
    } else {
        false
    }
}

/// Detects excessive character repetition (e.g., "aaaaaaa")
fn has_excessive_repetition(s: &str) -> bool {
    if s.len() < 4 {
        return false;
    }

    let max_repeat = 3;
    let mut prev = '\0';
    let mut count = 0;

    for ch in s.chars() {
        if ch == prev {
            count += 1;
            if count > max_repeat {
                return true;
            }
        } else {
            count = 1;
            prev = ch;
        }
    }

    false
}

/// Checks if image has any useful metadata (for OCR fallback decision)
fn has_any_useful_metadata(metadata: &FileMetadata) -> bool {
    is_useful_metadata(&metadata.title)
        || is_useful_metadata(&metadata.description)
        || is_useful_metadata(&metadata.date_time_original)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_useful_metadata_rejects_errors() {
        assert!(!is_useful_metadata(&Some("ERROR: Cannot read file".to_string())));
        assert!(!is_useful_metadata(&Some("Exception occurred".to_string())));
        assert!(!is_useful_metadata(&Some("Warning: Invalid data".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_rejects_devices() {
        assert!(!is_useful_metadata(&Some("Canon MX490".to_string())));
        assert!(!is_useful_metadata(&Some("HP LaserJet".to_string())));
        assert!(!is_useful_metadata(&Some("EPSON Scanner".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_rejects_generic() {
        assert!(!is_useful_metadata(&Some("Untitled Document".to_string())));
        assert!(!is_useful_metadata(&Some("New Document 1".to_string())));
        assert!(!is_useful_metadata(&Some("Copy of Report".to_string())));
        assert!(!is_useful_metadata(&Some("test_file".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_rejects_dates() {
        assert!(!is_useful_metadata(&Some("20231015".to_string())));
        assert!(!is_useful_metadata(&Some("2023-10-15".to_string())));
        assert!(!is_useful_metadata(&Some("202310".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_rejects_repetition() {
        assert!(!is_useful_metadata(&Some("aaaaaaa".to_string())));
        assert!(!is_useful_metadata(&Some("1111111".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_rejects_punctuation() {
        assert!(!is_useful_metadata(&Some("!!!###$$$".to_string())));
        assert!(!is_useful_metadata(&Some("---***---".to_string())));
    }

    #[test]
    fn test_is_useful_metadata_accepts_good_names() {
        assert!(is_useful_metadata(&Some("Quarterly Sales Report".to_string())));
        assert!(is_useful_metadata(&Some("Project Proposal".to_string())));
        assert!(is_useful_metadata(&Some("Meeting Notes Q3 2023".to_string())));
    }

    #[test]
    fn test_is_date_only() {
        use crate::scorer::is_date_only_pattern;

        assert!(is_date_only_pattern("20231015"));
        assert!(is_date_only_pattern("2023-10-15"));
        assert!(is_date_only_pattern("2023/10/15"));
        assert!(is_date_only_pattern("202310"));

        assert!(!is_date_only_pattern("Report 20231015"));
        assert!(!is_date_only_pattern("Q3 2023"));
        assert!(!is_date_only_pattern("abc123"));
    }

    #[test]
    fn test_has_excessive_repetition() {
        assert!(has_excessive_repetition("aaaaaaa"));
        assert!(has_excessive_repetition("1111111"));
        assert!(has_excessive_repetition("test----test"));

        assert!(!has_excessive_repetition("coffee"));  // double letters are ok
        assert!(!has_excessive_repetition("bookkeeper"));
        assert!(!has_excessive_repetition("aaa")); // exactly 3 is ok
    }
}
