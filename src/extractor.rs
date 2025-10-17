use anyhow::{Context, Result};
use log::debug;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::detector::FileCategory;
use crate::image_ocr;
use crate::pdf_content;
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
}

impl FileMetadata {
    /// Extracts the best candidate name from metadata based on file category
    pub fn extract_name(&self, category: &FileCategory) -> Option<String> {
        match category {
            FileCategory::Image => {
                self.title
                    .as_ref()
                    .or(self.description.as_ref())
                    .or(self.date_time_original.as_ref())
                    .cloned()
            }
            FileCategory::Document => {
                self.title
                    .as_ref()
                    .or(self.subject.as_ref())
                    .or(self.author.as_ref())
                    .cloned()
            }
            FileCategory::Audio => {
                self.title
                    .as_ref()
                    .or(self.artist.as_ref())
                    .or(self.album.as_ref())
                    .cloned()
            }
            FileCategory::Video => {
                self.title
                    .as_ref()
                    .or(self.creation_date.as_ref())
                    .cloned()
            }
            FileCategory::Unknown => None,
        }
    }
}

/// Extracts metadata from a file using exiftool
pub fn extract_metadata(path: &Path) -> Result<FileMetadata> {
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
    }

    let parsed: Vec<ExiftoolOutput> = serde_json::from_str(&json_str)
        .context("Failed to parse exiftool JSON output")?;

    let exif_data = parsed
        .into_iter()
        .next()
        .context("No metadata found in exiftool output")?;

    let author = exif_data.author
        .or(exif_data.creator)
        .or(exif_data.last_modified_by);

    // Filter out unhelpful author names (like scanner/printer names)
    let filtered_author = if is_useful_metadata(&author) {
        author
    } else {
        None
    };

    let mut metadata = FileMetadata {
        title: exif_data.title,
        artist: exif_data.artist,
        album: exif_data.album,
        date_time_original: exif_data.date_time_original,
        description: exif_data.description,
        subject: exif_data.subject,
        author: filtered_author,
        creation_date: exif_data.creation_date.or(exif_data.create_date),
    };

    // For PDFs without useful metadata, try extracting text content
    if is_pdf(path) && !is_useful_metadata(&metadata.title) && !is_useful_metadata(&metadata.subject) {
        debug!("PDF has no useful metadata, attempting content extraction");
        if let Ok(Some(content)) = pdf_content::extract_pdf_content(path) {
            debug!("Extracted PDF content: {}", content);
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
    if is_video(path) && !is_useful_metadata(&metadata.title) && !is_useful_metadata(&metadata.creation_date) {
        debug!("Video has no useful metadata, attempting frame extraction and OCR");
        if let Ok(Some(text)) = video_ocr::extract_video_text(path) {
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

/// Checks if metadata string is useful (not empty, not just scanner/printer names)
fn is_useful_metadata(value: &Option<String>) -> bool {
    if let Some(v) = value {
        let lower = v.to_lowercase();
        // Skip common unhelpful metadata
        if lower.is_empty()
            || lower.contains("canon")
            || lower.contains("printer")
            || lower.contains("scanner")
            || lower.contains("ipr")
            || lower.contains("untitled")
            || lower.len() < 3 {
            return false;
        }
        true
    } else {
        false
    }
}

/// Checks if image has any useful metadata (for OCR fallback decision)
fn has_any_useful_metadata(metadata: &FileMetadata) -> bool {
    is_useful_metadata(&metadata.title)
        || is_useful_metadata(&metadata.description)
        || is_useful_metadata(&metadata.date_time_original)
}
