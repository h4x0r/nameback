use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

use crate::detector::FileCategory;

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

    Ok(FileMetadata {
        title: exif_data.title,
        artist: exif_data.artist,
        album: exif_data.album,
        date_time_original: exif_data.date_time_original,
        description: exif_data.description,
        subject: exif_data.subject,
        author: exif_data.author
            .or(exif_data.creator)
            .or(exif_data.last_modified_by),
        creation_date: exif_data.creation_date.or(exif_data.create_date),
    })
}
