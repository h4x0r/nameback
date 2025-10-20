use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Represents the category of a file based on its type
#[derive(Debug, Clone, PartialEq)]
pub enum FileCategory {
    Image,
    Document,
    Audio,
    Video,
    Email,
    Web,
    Archive,
    SourceCode,
    Unknown,
}

/// Detects the file type using the infer library (pure Rust, cross-platform)
pub fn detect_file_type(path: &Path) -> Result<FileCategory> {
    // Read the first 8192 bytes for file type detection
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; 8192];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Use infer to detect file type from magic bytes
    let category = if let Some(kind) = infer::get(&buffer) {
        let mime_type = kind.mime_type();

        match mime_type {
            // Image types
            s if s.starts_with("image/") => FileCategory::Image,

            // Document types
            "application/pdf" => FileCategory::Document,
            s if s.starts_with("application/vnd.openxmlformats-officedocument") => {
                FileCategory::Document
            }
            s if s.starts_with("application/vnd.ms-") => FileCategory::Document,
            s if s.starts_with("application/vnd.oasis.opendocument") => FileCategory::Document,
            "application/rtf" => FileCategory::Document,
            "application/msword" => FileCategory::Document,
            s if s.starts_with("text/") => FileCategory::Document,

            // Audio types
            s if s.starts_with("audio/") => FileCategory::Audio,

            // Video types
            s if s.starts_with("video/") => FileCategory::Video,

            _ => FileCategory::Unknown,
        }
    } else {
        // Fallback to extension-based detection if magic bytes don't match
        detect_by_extension(path)
    };

    Ok(category)
}

/// Fallback file type detection based on extension
fn detect_by_extension(path: &Path) -> FileCategory {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                // Images
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" | "heic"
                | "heif" | "ico" | "svg" => FileCategory::Image,
                // Documents
                "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods"
                | "odp" | "rtf" | "txt" | "md" | "markdown" | "csv" => FileCategory::Document,
                // Email
                "eml" | "msg" => FileCategory::Email,
                // Web
                "html" | "htm" | "mhtml" => FileCategory::Web,
                // Archive
                "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => FileCategory::Archive,
                // Source Code (non-text mime types)
                "py" | "js" | "ts" | "rs" | "java" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" => FileCategory::SourceCode,
                // Config files as documents
                "json" | "yaml" | "yml" => FileCategory::Document,
                // Audio
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => {
                    FileCategory::Audio
                }
                // Video
                "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg" => {
                    FileCategory::Video
                }
                _ => FileCategory::Unknown,
            }
        })
        .unwrap_or(FileCategory::Unknown)
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
