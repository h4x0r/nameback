use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Represents the category of a file based on its type
#[derive(Debug, Clone, PartialEq)]
pub enum FileCategory {
    Image,
    Document,
    Audio,
    Video,
    Unknown,
}

/// Detects the file type using the `file` command
pub fn detect_file_type(path: &Path) -> Result<FileCategory> {
    let output = Command::new("file")
        .arg("-b") // Brief mode
        .arg(path)
        .output()
        .context("Failed to execute `file` command")?;

    if !output.status.success() {
        anyhow::bail!("file command failed with status: {}", output.status);
    }

    let file_type = String::from_utf8_lossy(&output.stdout).to_lowercase();

    let category = match file_type {
        s if s.contains("image") || s.contains("jpeg") || s.contains("png") || s.contains("gif") => {
            FileCategory::Image
        }
        s if s.contains("pdf") || s.contains("document") || s.contains("text") => {
            FileCategory::Document
        }
        s if s.contains("audio") || s.contains("mp3") || s.contains("flac") || s.contains("wav") => {
            FileCategory::Audio
        }
        s if s.contains("video") || s.contains("mp4") || s.contains("avi") || s.contains("mkv") => {
            FileCategory::Video
        }
        _ => FileCategory::Unknown,
    };

    Ok(category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_file_type_exists() {
        // Test that the function exists and returns a Result
        let test_path = PathBuf::from("/dev/null");
        let _ = detect_file_type(&test_path);
    }
}
