use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use std::process::Command;

/// Extracts text from a video by extracting a frame and running OCR
/// (requires ffmpeg and tesseract-ocr installed)
pub fn extract_video_text(path: &Path) -> Result<Option<String>> {
    debug!("Attempting video frame OCR on: {}", path.display());

    // Check if ffmpeg is available
    if !is_ffmpeg_available() {
        debug!("ffmpeg not available, skipping video OCR");
        return Ok(None);
    }

    // Check if tesseract is available
    if !is_tesseract_available() {
        debug!("Tesseract not available, skipping video OCR");
        return Ok(None);
    }

    // Extract a frame from the video
    let frame_path = match extract_video_frame(path) {
        Ok(frame) => frame,
        Err(e) => {
            debug!("Failed to extract video frame: {}", e);
            return Ok(None);
        }
    };

    // Run OCR on the frame
    let result = run_tesseract_ocr(&frame_path);

    // Clean up temp frame file
    let _ = std::fs::remove_file(&frame_path);

    match result {
        Ok(text) => {
            let cleaned = clean_text(&text);
            if cleaned.len() > 10 {
                let truncated = if cleaned.len() > 80 {
                    &cleaned[..80]
                } else {
                    &cleaned
                };
                debug!("Video OCR extracted: {}", truncated);
                Ok(Some(truncated.to_string()))
            } else {
                debug!("Video OCR text too short");
                Ok(None)
            }
        }
        Err(e) => {
            debug!("Video OCR failed: {}", e);
            Ok(None)
        }
    }
}

/// Extracts text from multiple video frames and selects the best result
/// Tries frames at 1s, 5s, and 10s, then scores each result for quality
pub fn extract_video_text_multiframe(path: &Path) -> Result<Option<String>> {
    use crate::scorer::{NameCandidate, NameSource};

    debug!("Attempting multi-frame video OCR on: {}", path.display());

    // Check if tools are available
    if !is_ffmpeg_available() {
        debug!("ffmpeg not available, skipping video OCR");
        return Ok(None);
    }

    if !is_tesseract_available() {
        debug!("Tesseract not available, skipping video OCR");
        return Ok(None);
    }

    // Try multiple frame positions
    let frame_times = ["00:00:01", "00:00:05", "00:00:10"];
    let mut candidates = Vec::new();

    for time in &frame_times {
        debug!("Extracting frame at {}", time);

        match extract_video_frame_at_time(path, time) {
            Ok(frame_path) => {
                match run_tesseract_ocr(&frame_path) {
                    Ok(text) => {
                        let cleaned = clean_text(&text);
                        if cleaned.len() > 10 {
                            let truncated = if cleaned.len() > 80 {
                                &cleaned[..80]
                            } else {
                                &cleaned
                            };
                            debug!("Frame at {} extracted: {}", time, truncated);
                            candidates.push(NameCandidate::new(
                                truncated.to_string(),
                                NameSource::OcrVideo,
                            ));
                        }
                    }
                    Err(e) => {
                        debug!("OCR failed for frame at {}: {}", time, e);
                    }
                }

                // Clean up temp frame
                let _ = std::fs::remove_file(&frame_path);
            }
            Err(e) => {
                debug!("Failed to extract frame at {}: {}", time, e);
            }
        }
    }

    // Select best candidate using scorer
    if let Some(best) = crate::scorer::select_best_candidate(candidates) {
        debug!("Selected best video OCR result with score: {}", best.score);
        Ok(Some(best.name))
    } else {
        debug!("No suitable video OCR text found");
        Ok(None)
    }
}

/// Extracts a frame at a specific time from the video
fn extract_video_frame_at_time(video_path: &Path, time: &str) -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let temp_frame = temp_dir.join(format!(
        "nameback_video_{}_{}.png",
        std::process::id(),
        time.replace(':', "_")
    ));

    debug!(
        "Extracting frame from video: {} at {} -> {}",
        video_path.display(),
        time,
        temp_frame.display()
    );

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path)
        .arg("-ss")
        .arg(time)
        .arg("-vframes")
        .arg("1")
        .arg("-f")
        .arg("image2")
        .arg(&temp_frame)
        .arg("-y")
        .output()
        .context("Failed to run ffmpeg command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr);
    }

    if !temp_frame.exists() {
        anyhow::bail!("Frame extraction succeeded but file not found");
    }

    debug!("Successfully extracted video frame at {}", time);
    Ok(temp_frame)
}

/// Checks if ffmpeg is installed and available
fn is_ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Checks if tesseract-ocr is installed and available
fn is_tesseract_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Extracts a single frame from a video file using ffmpeg
/// Extracts frame at 1 second into the video
fn extract_video_frame(video_path: &Path) -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let temp_frame = temp_dir.join(format!("nameback_video_{}.png", std::process::id()));

    debug!(
        "Extracting frame from video: {} -> {}",
        video_path.display(),
        temp_frame.display()
    );

    // Extract frame at 1 second mark
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path)
        .arg("-ss")
        .arg("00:00:01") // 1 second into the video
        .arg("-vframes")
        .arg("1") // Extract 1 frame
        .arg("-f")
        .arg("image2")
        .arg(&temp_frame)
        .arg("-y") // Overwrite if exists
        .output()
        .context("Failed to run ffmpeg command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr);
    }

    // Verify the frame was created
    if !temp_frame.exists() {
        anyhow::bail!("Frame extraction succeeded but file not found");
    }

    debug!("Successfully extracted video frame");
    Ok(temp_frame)
}

/// Runs tesseract OCR on an image file
/// Tries multiple languages in priority order: Traditional Chinese, Simplified Chinese, English
fn run_tesseract_ocr(image_path: &Path) -> Result<String> {
    let path_str = image_path.to_str().context("Path not valid UTF-8")?;

    // Try languages in order: Traditional Chinese, Simplified Chinese, English
    let languages = ["chi_tra", "chi_sim", "eng"];
    let mut best_result = String::new();
    let mut best_confidence = 0;

    for lang in &languages {
        debug!("Trying video OCR with language: {}", lang);

        let result = tesseract::Tesseract::new(None, Some(lang))
            .context("Failed to initialize Tesseract")
            .and_then(|t| t.set_image(path_str).context("Failed to set image"))
            .and_then(|mut t| t.get_text().context("Failed to extract text"));

        match result {
            Ok(text) => {
                let cleaned = clean_text(&text);
                let char_count = cleaned.chars().count();

                debug!(
                    "Video OCR with {}: {} characters extracted",
                    lang, char_count
                );

                // Use the result with the most characters as proxy for best match
                if char_count > best_confidence {
                    best_confidence = char_count;
                    best_result = text;
                    debug!("New best result with {}: {} chars", lang, char_count);
                }
            }
            Err(e) => {
                debug!("Video OCR with {} failed: {}", lang, e);
            }
        }
    }

    if best_confidence > 0 {
        Ok(best_result)
    } else {
        anyhow::bail!("All video OCR language attempts failed")
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
        let input = "Video   Title\n\nEpisode  One\n  Scene  1";
        let expected = "Video Title Episode One Scene 1";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_clean_text_empty() {
        let input = "\n\n   \n  ";
        assert_eq!(clean_text(input), "");
    }
}
