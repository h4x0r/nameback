//! Homebrew package manager integration for macOS
//!
//! Handles dependency installation on macOS using Homebrew as primary
//! and MacPorts as fallback package manager.

use std::process::Command;

/// Installs all dependencies on macOS via Homebrew/MacPorts
///
/// Installs dependencies in order:
/// 1. exiftool (required - core metadata extraction)
/// 2. tesseract (optional - OCR support)
/// 3. ffmpeg (optional - video frame extraction)
/// 4. imagemagick (optional - HEIC/HEIF support)
///
/// # Arguments
/// * `report_progress` - Progress callback for UI updates
///
/// # Returns
/// * `Ok(())` if required dependencies installed successfully
/// * `Err(String)` if required dependencies failed
pub fn install_dependencies(
    report_progress: &impl Fn(&str, u8)
) -> Result<(), String> {
    report_progress("Checking Homebrew installation...", 10);

    // Check if Homebrew is installed
    let brew_check = Command::new("brew")
        .arg("--version")
        .output();

    let brew_installed = brew_check
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !brew_installed {
        println!("Homebrew not found. Checking for MacPorts as fallback...");

        // Try MacPorts as fallback
        let port_check = Command::new("port")
            .arg("version")
            .output();

        let port_installed = port_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !port_installed {
            return Err(
                "No package manager found. Please install Homebrew (https://brew.sh) or MacPorts (https://www.macports.org)".to_string()
            );
        }

        println!("MacPorts found, using as fallback package manager");
    }

    // Helper to install with Homebrew with DNS fallback
    let install_with_brew = |package: &str| -> bool {
        println!("Installing {} with Homebrew...", package);
        let result = Command::new("brew")
            .args(["install", package])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                println!("{} installed successfully", package);
                true
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("Homebrew installation failed for {}: {}", package, stderr);

                // Check for DNS/network errors
                if stderr.contains("Could not resolve") ||
                   stderr.contains("Failed to connect") ||
                   stderr.contains("curl") && stderr.contains("error") {
                    println!("Detected network error, trying DNS fallback...");

                    if super::try_with_public_dns().is_ok() {
                        println!("Retrying {} installation with public DNS...", package);
                        let retry = Command::new("brew")
                            .args(["install", package])
                            .output();

                        super::restore_dns();

                        if let Ok(retry_output) = retry {
                            if retry_output.status.success() {
                                println!("{} installed successfully with DNS fallback", package);
                                return true;
                            }
                        }
                    }
                }
                false
            }
            Err(e) => {
                println!("Failed to execute brew command: {}", e);
                false
            }
        }
    };

    // Helper to install with MacPorts as fallback
    let install_with_port = |package: &str| -> bool {
        println!("Trying MacPorts as fallback for {}...", package);
        let result = Command::new("sudo")
            .args(["port", "install", package])
            .status();

        match result {
            Ok(status) if status.success() => {
                println!("{} installed successfully via MacPorts", package);
                true
            }
            _ => {
                println!("MacPorts installation also failed for {}", package);
                false
            }
        }
    };

    // Install exiftool (required)
    report_progress("Installing exiftool (required)...", 30);

    let exiftool_installed = if brew_installed {
        install_with_brew("exiftool") || install_with_port("exiftool")
    } else {
        install_with_port("exiftool")
    };

    if !exiftool_installed {
        super::restore_dns();
        return Err("Failed to install exiftool. Please install manually: brew install exiftool".to_string());
    }

    // Install tesseract (optional)
    report_progress("Installing tesseract (optional OCR support)...", 50);
    let tesseract_installed = if brew_installed {
        install_with_brew("tesseract") || install_with_brew("tesseract-lang")
    } else {
        install_with_port("tesseract")
    };

    if !tesseract_installed {
        println!("WARNING: Tesseract (OCR) installation failed (optional)");
        println!("  OCR support will be disabled");
        println!("  Install manually: brew install tesseract tesseract-lang");
    }

    // Install ffmpeg (optional)
    report_progress("Installing ffmpeg (optional video support)...", 70);
    let ffmpeg_installed = if brew_installed {
        install_with_brew("ffmpeg")
    } else {
        install_with_port("ffmpeg")
    };

    if !ffmpeg_installed {
        println!("WARNING: FFmpeg installation failed (optional)");
        println!("  Video frame extraction will be disabled");
        println!("  Install manually: brew install ffmpeg");
    }

    // Install imagemagick (optional)
    report_progress("Installing imagemagick (optional HEIC support)...", 90);
    let imagemagick_installed = if brew_installed {
        install_with_brew("imagemagick")
    } else {
        install_with_port("ImageMagick")
    };

    if !imagemagick_installed {
        println!("WARNING: ImageMagick installation failed (optional)");
        println!("  HEIC/HEIF image support will be disabled");
        println!("  Install manually: brew install imagemagick");
    }

    report_progress("macOS dependencies installed", 100);
    Ok(())
}
