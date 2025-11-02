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
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_detect_by_extension_images() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("photo.jpg")),
            FileCategory::Image
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("image.png")),
            FileCategory::Image
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("graphic.gif")),
            FileCategory::Image
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("photo.HEIC")),
            FileCategory::Image
        );
    }

    #[test]
    fn test_detect_by_extension_documents() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("report.pdf")),
            FileCategory::Document
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("doc.docx")),
            FileCategory::Document
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("sheet.xlsx")),
            FileCategory::Document
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("notes.txt")),
            FileCategory::Document
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("config.json")),
            FileCategory::Document
        );
    }

    #[test]
    fn test_detect_by_extension_audio() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("song.mp3")),
            FileCategory::Audio
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("audio.wav")),
            FileCategory::Audio
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("track.flac")),
            FileCategory::Audio
        );
    }

    #[test]
    fn test_detect_by_extension_video() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("movie.mp4")),
            FileCategory::Video
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("clip.avi")),
            FileCategory::Video
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("video.mkv")),
            FileCategory::Video
        );
    }

    #[test]
    fn test_detect_by_extension_email() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("message.eml")),
            FileCategory::Email
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("email.msg")),
            FileCategory::Email
        );
    }

    #[test]
    fn test_detect_by_extension_web() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("page.html")),
            FileCategory::Web
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("site.htm")),
            FileCategory::Web
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("archive.mhtml")),
            FileCategory::Web
        );
    }

    #[test]
    fn test_detect_by_extension_archive() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("files.zip")),
            FileCategory::Archive
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("backup.tar")),
            FileCategory::Archive
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("archive.gz")),
            FileCategory::Archive
        );
    }

    #[test]
    fn test_detect_by_extension_source_code() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("script.py")),
            FileCategory::SourceCode
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("app.js")),
            FileCategory::SourceCode
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("main.rs")),
            FileCategory::SourceCode
        );
    }

    #[test]
    fn test_detect_by_extension_unknown() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("unknown.xyz")),
            FileCategory::Unknown
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("noextension")),
            FileCategory::Unknown
        );
    }

    #[test]
    fn test_detect_by_extension_case_insensitive() {
        assert_eq!(
            detect_by_extension(&PathBuf::from("IMAGE.JPG")),
            FileCategory::Image
        );
        assert_eq!(
            detect_by_extension(&PathBuf::from("Document.PDF")),
            FileCategory::Document
        );
    }

    #[test]
    fn test_detect_file_type_with_temp_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a simple PNG file (PNG magic bytes)
        let png_path = temp_dir.path().join("test.png");
        let png_magic = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        fs::write(&png_path, png_magic).unwrap();

        let result = detect_file_type(&png_path).unwrap();
        assert_eq!(result, FileCategory::Image);
    }

    #[test]
    fn test_detect_file_type_jpeg() {
        let temp_dir = TempDir::new().unwrap();

        // Create a JPEG file (JPEG magic bytes: FF D8 FF)
        let jpeg_path = temp_dir.path().join("test.jpg");
        let jpeg_magic = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        fs::write(&jpeg_path, jpeg_magic).unwrap();

        let result = detect_file_type(&jpeg_path).unwrap();
        assert_eq!(result, FileCategory::Image);
    }

    #[test]
    fn test_detect_file_type_pdf() {
        let temp_dir = TempDir::new().unwrap();

        // Create a PDF file (PDF magic bytes: %PDF)
        let pdf_path = temp_dir.path().join("test.pdf");
        let pdf_magic = b"%PDF-1.4\n".to_vec();
        fs::write(&pdf_path, pdf_magic).unwrap();

        let result = detect_file_type(&pdf_path).unwrap();
        assert_eq!(result, FileCategory::Document);
    }

    #[test]
    fn test_detect_file_type_falls_back_to_extension() {
        let temp_dir = TempDir::new().unwrap();

        // Create a text file with no magic bytes
        let txt_path = temp_dir.path().join("test.txt");
        fs::write(&txt_path, "Hello, world!").unwrap();

        let result = detect_file_type(&txt_path).unwrap();
        assert_eq!(result, FileCategory::Document);
    }
}
