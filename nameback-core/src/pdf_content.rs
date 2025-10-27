use anyhow::{Context, Result};
use image::DynamicImage;
use log::debug;
use std::path::Path;
use std::process::Command;

/// Extracts text content from a PDF file and returns the first useful portion
pub fn extract_pdf_content(path: &Path) -> Result<Option<String>> {
    // Try extracting text from PDF first
    match pdf_extract::extract_text(path) {
        Ok(text) => {
            // For PDFs, prioritize the beginning where titles typically appear
            // Extract from RAW text BEFORE cleaning (which collapses lines)
            let raw_lines: Vec<&str> = text
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty() && line.len() > 3)
                .take(4)  // Get first 4 non-empty lines
                .collect();

            // Combine first few lines to build a comprehensive title
            if !raw_lines.is_empty() {
                // Try combining short, meaningful lines (skip overly long subtitle lines)
                let mut combined = String::new();
                for line in &raw_lines {
                    // Skip lines that are too long (likely subtitles/taglines)
                    // unless they're the first line
                    if combined.is_empty() {
                        combined = line.to_string();
                    } else if line.len() <= 30 {
                        // Only add short lines to avoid subtitle clutter
                        let test = format!("{} {}", combined, line);
                        // Stop if we'd exceed 80 chars
                        if test.len() > 80 {
                            break;
                        }
                        combined = test;
                    }
                    // If we have a good length (30+ chars), we have enough context
                    if combined.len() >= 30 {
                        break;
                    }
                }

                // Accept the combined title if it's at least 10 chars
                if combined.len() >= 10 {
                    debug!("Using combined title from document start: {}", combined);
                    return Ok(Some(combined));
                }
            }

            // Fallback: clean the text and use key phrase extraction
            let cleaned = clean_text(&text);
            if cleaned.len() > 150 {
                debug!("Extracting key phrases from PDF text ({} chars)", cleaned.len());
                let phrases = crate::key_phrases::extract_key_phrases(&cleaned, 3);
                if !phrases.is_empty() {
                    let best_phrase = &phrases[0];
                    debug!("Selected key phrase: {}", best_phrase);
                    return Ok(Some(best_phrase.clone()));
                }
            }

            // Final fallback: truncate from beginning of cleaned text
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
                // Use key phrase extraction for longer OCR text
                if cleaned.len() > 150 {
                    debug!("Extracting key phrases from OCR text ({} chars)", cleaned.len());
                    let phrases = crate::key_phrases::extract_key_phrases(&cleaned, 3);
                    if !phrases.is_empty() {
                        let best_phrase = &phrases[0];
                        debug!("Selected key phrase from OCR: {}", best_phrase);
                        return Ok(Some(best_phrase.clone()));
                    }
                }

                // For shorter text or if key phrase extraction failed, truncate
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
    let img = image::open(&image_path).context("Failed to open generated PNG")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&image_path);

    Ok(img)
}

/// Runs tesseract OCR on an image
/// Tries multiple languages in priority order: Traditional Chinese, Simplified Chinese, English
fn run_tesseract_ocr(image: &DynamicImage) -> Result<String> {
    // Save image to temp file for tesseract
    let temp_dir = std::env::temp_dir();
    let temp_img = temp_dir.join(format!("nameback_ocr_{}.png", std::process::id()));

    image.save(&temp_img).context("Failed to save temp image")?;

    let temp_img_str = temp_img.to_str().context("Path not valid UTF-8")?;

    // Try languages in order: Traditional Chinese, Simplified Chinese, English
    let languages = ["chi_tra", "chi_sim", "eng"];
    let mut best_result = String::new();
    let mut best_confidence = 0;

    for lang in &languages {
        debug!("Trying OCR with language: {}", lang);

        let result = tesseract::Tesseract::new(None, Some(lang))
            .context("Failed to initialize Tesseract")
            .and_then(|t| t.set_image(temp_img_str).context("Failed to set image"))
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

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_img);

    if best_confidence > 0 {
        Ok(best_result)
    } else {
        anyhow::bail!("All OCR language attempts failed")
    }
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
