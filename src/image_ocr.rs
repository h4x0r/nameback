use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use std::process::Command;

/// Extracts text from an image using OCR (requires tesseract-ocr installed)
pub fn extract_image_text(path: &Path) -> Result<Option<String>> {
    debug!("Attempting OCR on image: {}", path.display());

    // Check if tesseract is available
    if !is_tesseract_available() {
        debug!("Tesseract not available, skipping OCR");
        return Ok(None);
    }

    // Run tesseract OCR on the image
    match run_tesseract_ocr(path) {
        Ok(text) => {
            let cleaned = clean_text(&text);
            if cleaned.len() > 10 {
                let truncated = if cleaned.len() > 80 {
                    &cleaned[..80]
                } else {
                    &cleaned
                };
                debug!("OCR extracted from image: {}", truncated);
                Ok(Some(truncated.to_string()))
            } else {
                debug!("OCR text too short");
                Ok(None)
            }
        }
        Err(e) => {
            debug!("OCR failed: {}", e);
            Ok(None)
        }
    }
}

/// Checks if tesseract-ocr is installed and available
fn is_tesseract_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Runs tesseract OCR on an image file
/// Tries multiple languages in priority order: Traditional Chinese, Simplified Chinese, English
fn run_tesseract_ocr(image_path: &Path) -> Result<String> {
    // Convert to absolute path
    let absolute_path = if image_path.is_absolute() {
        image_path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .join(image_path)
    };

    // Check if this is a HEIC file that needs conversion
    let needs_conversion = image_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "heic" | "heif"))
        .unwrap_or(false);

    // Convert HEIC to PNG if needed
    let (ocr_path, temp_file) = if needs_conversion {
        let temp_png = convert_heic_to_png(&absolute_path)?;
        (temp_png.clone(), Some(temp_png))
    } else {
        (absolute_path.clone(), None)
    };

    let path_str = ocr_path.to_str().context("Path not valid UTF-8")?;

    // Try languages in order: Traditional Chinese, Simplified Chinese, English
    let languages = ["chi_tra", "chi_sim", "eng"];
    let mut best_result = String::new();
    let mut best_confidence = 0;

    for lang in &languages {
        debug!("Trying OCR with language: {}", lang);

        let result = tesseract::Tesseract::new(None, Some(lang))
            .context("Failed to initialize Tesseract")
            .and_then(|t| t.set_image(path_str).context("Failed to set image"))
            .and_then(|mut t| t.get_text().context("Failed to extract text"));

        match result {
            Ok(text) => {
                let cleaned = clean_text(&text);
                let char_count = cleaned.chars().count();

                debug!("OCR with {}: {} characters extracted", lang, char_count);

                // Use the result with the most characters as proxy for best match
                if char_count > best_confidence {
                    best_confidence = char_count;
                    best_result = text;
                    debug!("New best result with {}: {} chars", lang, char_count);
                }
            }
            Err(e) => {
                debug!("OCR with {} failed: {}", lang, e);
            }
        }
    }

    // Clean up temp file if we created one
    if let Some(temp) = temp_file {
        let _ = std::fs::remove_file(&temp);
    }

    if best_confidence > 0 {
        Ok(best_result)
    } else {
        anyhow::bail!("All OCR language attempts failed")
    }
}

/// Converts HEIC image to PNG using sips (macOS) or magick (ImageMagick)
fn convert_heic_to_png(heic_path: &Path) -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let temp_png = temp_dir.join(format!("nameback_heic_{}.png", std::process::id()));

    debug!(
        "Converting HEIC to PNG: {} -> {}",
        heic_path.display(),
        temp_png.display()
    );

    // Try sips first (available on macOS)
    let sips_result = Command::new("sips")
        .arg("-s")
        .arg("format")
        .arg("png")
        .arg(heic_path)
        .arg("--out")
        .arg(&temp_png)
        .output();

    if let Ok(output) = sips_result {
        if output.status.success() {
            debug!("Successfully converted HEIC using sips");
            return Ok(temp_png);
        }
    }

    // Fallback to ImageMagick's magick command
    debug!("sips not available or failed, trying ImageMagick");
    let output = Command::new("magick")
        .arg("convert")
        .arg(heic_path)
        .arg(&temp_png)
        .output()
        .context("Failed to run magick command - neither sips nor ImageMagick available")?;

    if !output.status.success() {
        anyhow::bail!("HEIC conversion failed with both sips and magick");
    }

    debug!("Successfully converted HEIC using ImageMagick");
    Ok(temp_png)
}

/// Cleans extracted text for use in filenames
fn clean_text(text: &str) -> String {
    // Remove excessive whitespace and newlines
    let cleaned = text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Collapse multiple spaces
    let mut result = String::new();
    let mut last_was_space = false;

    for ch in cleaned.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Error   Message\n\nDatabase  Connection\n  Failed";
        let expected = "Error Message Database Connection Failed";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_clean_text_empty() {
        let input = "\n\n   \n  ";
        assert_eq!(clean_text(input), "");
    }
}
