use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
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

    /// Find the executable path for this dependency
    /// Returns Some(path) if found, None otherwise
    pub fn find_executable(&self) -> Option<PathBuf> {
        find_tool_path(self.name(), self.fallback_names())
    }

    /// Create a Command for this dependency
    /// Returns None if the tool is not available
    pub fn create_command(&self) -> Option<Command> {
        self.find_executable().map(Command::new)
    }

    /// Check if this dependency is available
    pub fn is_available(&self) -> bool {
        if let Some(mut cmd) = self.create_command() {
            // Try to run version check
            let result = match self {
                Dependency::ExifTool => cmd.arg("-ver").output(),
                Dependency::Tesseract => cmd.arg("--version").output(),
                Dependency::FFmpeg => cmd.arg("-version").output(),
                Dependency::ImageMagick => cmd.arg("-version").output(),
            };

            let available = result.map(|o| o.status.success()).unwrap_or(false);
            log::debug!("Dependency check - {}: {}", self.name(),
                       if available { "available" } else { "missing" });
            available
        } else {
            log::debug!("Dependency check - {}: missing", self.name());
            false
        }
    }

    /// Get fallback executable names (for ImageMagick which can be "convert" on Linux/macOS)
    fn fallback_names(&self) -> &[&str] {
        match self {
            Dependency::ImageMagick => &["magick", "convert"],
            _ => &[],
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
    let mut needs_imagemagick = false; // Reserved for future HEIC detection

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

/// Unified helper to find a tool's executable path
/// Checks PATH first, then Scoop shims on Windows, supports fallback names
fn find_tool_path(primary_name: &str, fallback_names: &[&str]) -> Option<PathBuf> {
    // Try primary name in PATH
    if which::which(primary_name).is_ok() {
        log::debug!("Found {} in PATH", primary_name);
        return Some(PathBuf::from(primary_name));
    }

    // Try fallback names in PATH (e.g., "convert" for ImageMagick)
    for name in fallback_names {
        if which::which(name).is_ok() {
            log::debug!("Found {} in PATH (fallback for {})", name, primary_name);
            return Some(PathBuf::from(name));
        }
    }

    // On Windows, check Scoop shims directory
    // This is needed because PATH changes don't take effect until process restart
    #[cfg(windows)]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let scoop_shims = PathBuf::from(&userprofile).join("scoop").join("shims");

            // Check primary name
            let primary_path = scoop_shims.join(format!("{}.exe", primary_name));
            if primary_path.exists() {
                log::debug!("Found {} in Scoop shims: {:?}", primary_name, primary_path);
                return Some(primary_path);
            }

            // Check fallback names
            for name in fallback_names {
                let fallback_path = scoop_shims.join(format!("{}.exe", name));
                if fallback_path.exists() {
                    log::debug!("Found {} in Scoop shims (fallback for {}): {:?}",
                               name, primary_name, fallback_path);
                    return Some(fallback_path);
                }
            }
        }
    }

    log::debug!("Tool not found: {}", primary_name);
    None
}

/// Legacy helper for backward compatibility
/// Prefer using Dependency::create_command() instead
pub fn create_command(tool_name: &str) -> Command {
    find_tool_path(tool_name, &[])
        .map(Command::new)
        .unwrap_or_else(|| Command::new(tool_name))
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
