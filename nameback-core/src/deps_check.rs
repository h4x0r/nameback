use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Represents a dependency that might be needed
#[derive(Debug, Clone, PartialEq)]
pub enum Dependency {
    ExifTool,
    Tesseract,
    FFmpeg,
    ImageMagick,
}

impl Dependency {
    pub fn name(&self) -> &str {
        match self {
            Dependency::ExifTool => "exiftool",
            Dependency::Tesseract => "tesseract",
            Dependency::FFmpeg => "ffmpeg",
            Dependency::ImageMagick => "imagemagick",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Dependency::ExifTool => "Core metadata extraction (required)",
            Dependency::Tesseract => "OCR for images and videos",
            Dependency::FFmpeg => "Video frame extraction",
            Dependency::ImageMagick => "HEIC/HEIF support",
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            Dependency::ExifTool => check_exiftool(),
            Dependency::Tesseract => check_tesseract(),
            Dependency::FFmpeg => check_ffmpeg(),
            Dependency::ImageMagick => check_imagemagick(),
        }
    }
}

/// Result of smart dependency detection
#[derive(Debug)]
pub struct DependencyNeeds {
    pub missing_required: Vec<Dependency>,
    pub missing_optional: Vec<Dependency>,
}

impl DependencyNeeds {
    pub fn is_empty(&self) -> bool {
        self.missing_required.is_empty() && self.missing_optional.is_empty()
    }

    pub fn has_required_missing(&self) -> bool {
        !self.missing_required.is_empty()
    }
}

/// Smart detection: scan directory and determine which dependencies are actually needed
pub fn detect_needed_dependencies(directory: &Path) -> Result<DependencyNeeds> {
    let mut needs_tesseract = false;
    let mut needs_ffmpeg = false;
    let needs_imagemagick = false;

    // Quick scan of file types (just check extensions)
    let mut file_count = 0;
    for entry in WalkDir::new(directory)
        .max_depth(3) // Don't scan too deep
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        file_count += 1;
        if file_count > 1000 {
            // Sampled enough files
            break;
        }

        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();

            match ext_lower.as_str() {
                // Images that might need OCR
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "tif" | "webp" => {
                    needs_tesseract = true;
                }
                // HEIC files need ImageMagick on Windows/Linux
                "heic" | "heif" => {
                    needs_tesseract = true;
                    #[cfg(not(target_os = "macos"))]
                    {
                        needs_imagemagick = true;
                    }
                }
                // Videos need FFmpeg for frame extraction
                "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "wmv" | "m4v" => {
                    needs_ffmpeg = true;
                    needs_tesseract = true; // For OCR on extracted frames
                }
                _ => {}
            }
        }
    }

    // Check which dependencies are actually missing
    let mut missing_required = Vec::new();
    let mut missing_optional = Vec::new();

    // ExifTool is always required
    if !Dependency::ExifTool.is_available() {
        missing_required.push(Dependency::ExifTool);
    }

    // Optional dependencies - only if needed
    if needs_tesseract && !Dependency::Tesseract.is_available() {
        missing_optional.push(Dependency::Tesseract);
    }

    if needs_ffmpeg && !Dependency::FFmpeg.is_available() {
        missing_optional.push(Dependency::FFmpeg);
    }

    if needs_imagemagick && !Dependency::ImageMagick.is_available() {
        missing_optional.push(Dependency::ImageMagick);
    }

    Ok(DependencyNeeds {
        missing_required,
        missing_optional,
    })
}

// Helper functions to check if dependencies are available

fn check_exiftool() -> bool {
    std::process::Command::new("exiftool")
        .arg("-ver")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_tesseract() -> bool {
    std::process::Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_ffmpeg() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_imagemagick() -> bool {
    std::process::Command::new("magick")
        .arg("-version")
        .output()
        .or_else(|_| {
            std::process::Command::new("convert")
                .arg("-version")
                .output()
        })
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_names() {
        assert_eq!(Dependency::ExifTool.name(), "exiftool");
        assert_eq!(Dependency::Tesseract.name(), "tesseract");
        assert_eq!(Dependency::FFmpeg.name(), "ffmpeg");
        assert_eq!(Dependency::ImageMagick.name(), "imagemagick");
    }

    #[test]
    fn test_dependency_needs_empty() {
        let needs = DependencyNeeds {
            missing_required: vec![],
            missing_optional: vec![],
        };
        assert!(needs.is_empty());
        assert!(!needs.has_required_missing());
    }

    #[test]
    fn test_dependency_needs_with_required() {
        let needs = DependencyNeeds {
            missing_required: vec![Dependency::ExifTool],
            missing_optional: vec![],
        };
        assert!(!needs.is_empty());
        assert!(needs.has_required_missing());
    }

    #[test]
    fn test_dependency_needs_with_optional() {
        let needs = DependencyNeeds {
            missing_required: vec![],
            missing_optional: vec![Dependency::Tesseract],
        };
        assert!(!needs.is_empty());
        assert!(!needs.has_required_missing());
    }
}
