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
fn run_tesseract_ocr(image_path: &Path) -> Result<String> {
    // Convert to absolute path
    let absolute_path = if image_path.is_absolute() {
        image_path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("Failed to get current directory")?
            .join(image_path)
    };

    let text = tesseract::Tesseract::new(None, Some("eng"))
        .context("Failed to initialize Tesseract")?
        .set_image(absolute_path.to_str().context("Path not valid UTF-8")?)
        .context("Failed to set image")?
        .get_text()
        .context("Failed to extract text")?;

    Ok(text)
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
