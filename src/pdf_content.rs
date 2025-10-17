use anyhow::{Context, Result};
use image::DynamicImage;
use log::{debug, warn};
use std::path::Path;
use std::process::Command;

/// Extracts text content from a PDF file and returns the first useful portion
pub fn extract_pdf_content(path: &Path) -> Result<Option<String>> {
    // Try extracting text from PDF first
    match pdf_extract::extract_text(path) {
        Ok(text) => {
            let cleaned = clean_text(&text);
            if cleaned.len() > 10 {
                let truncated = if cleaned.len() > 80 {
                    &cleaned[..80]
                } else {
                    &cleaned
                };
                return Ok(Some(truncated.to_string()));
            }
            // Text too short, fall through to OCR
            debug!("PDF text too short ({} chars), trying OCR", cleaned.len());
        }
        Err(e) => {
            debug!("PDF text extraction failed: {}, trying OCR", e);
        }
    }

    // Fallback to OCR if text extraction failed or returned insufficient text
    extract_pdf_with_ocr(path)
}

/// Extracts text from PDF using OCR (requires tesseract-ocr installed)
fn extract_pdf_with_ocr(path: &Path) -> Result<Option<String>> {
    debug!("Attempting OCR on PDF: {}", path.display());

    // Check if tesseract is available
    if !is_tesseract_available() {
        debug!("Tesseract not available, skipping OCR");
        return Ok(None);
    }

    // Try to convert first page of PDF to image using pdftoppm
    let image = match pdf_page_to_image(path) {
        Ok(img) => img,
        Err(e) => {
            debug!("Failed to convert PDF to image: {}", e);
            return Ok(None);
        }
    };

    // Run OCR on the image
    match run_tesseract_ocr(&image) {
        Ok(text) => {
            let cleaned = clean_text(&text);
            if cleaned.len() > 10 {
                let truncated = if cleaned.len() > 80 {
                    &cleaned[..80]
                } else {
                    &cleaned
                };
                debug!("OCR extracted: {}", truncated);
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

/// Converts first page of PDF to image using pdftoppm
fn pdf_page_to_image(path: &Path) -> Result<DynamicImage> {
    // Create temporary directory for image output
    let temp_dir = std::env::temp_dir();
    let temp_prefix = temp_dir.join(format!("nameback_pdf_{}", std::process::id()));

    // Run pdftoppm to convert first page to PNG
    let output = Command::new("pdftoppm")
        .arg("-png")
        .arg("-f")
        .arg("1")
        .arg("-l")
        .arg("1")
        .arg("-singlefile")
        .arg(path)
        .arg(&temp_prefix)
        .output()
        .context("Failed to run pdftoppm - is poppler-utils installed?")?;

    if !output.status.success() {
        anyhow::bail!("pdftoppm failed");
    }

    // Load the generated image
    let image_path = temp_prefix.with_extension("png");
    let img = image::open(&image_path)
        .context("Failed to open generated PNG")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&image_path);

    Ok(img)
}

/// Runs tesseract OCR on an image
fn run_tesseract_ocr(image: &DynamicImage) -> Result<String> {
    // Save image to temp file for tesseract
    let temp_dir = std::env::temp_dir();
    let temp_img = temp_dir.join(format!("nameback_ocr_{}.png", std::process::id()));

    image.save(&temp_img)
        .context("Failed to save temp image")?;

    let text = tesseract::Tesseract::new(None, Some("eng"))
        .context("Failed to initialize Tesseract")?
        .set_image(temp_img.to_str().unwrap())
        .context("Failed to set image")?
        .get_text()
        .context("Failed to extract text")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_img);

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
        let input = "Annual   Report\n\n2024\n  Financial  Statement";
        let expected = "Annual Report 2024 Financial Statement";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_clean_text_empty() {
        let input = "\n\n   \n  ";
        assert_eq!(clean_text(input), "");
    }
}
