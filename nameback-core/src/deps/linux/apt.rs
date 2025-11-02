//! Linux package manager integration
//!
//! Handles dependency installation on Linux using multiple package managers:
//! apt-get (Debian/Ubuntu), dnf/yum (Fedora/RHEL), pacman (Arch), snap

use std::process::Command;

/// Installs all dependencies on Linux via available package managers
///
/// Detects and uses available package managers in this order:
/// 1. apt-get (Debian, Ubuntu, Kali)
/// 2. dnf (Fedora, RHEL 8+)
/// 3. yum (RHEL 7, CentOS 7)
/// 4. pacman (Arch, Manjaro)
/// 5. snap (Universal)
///
/// Installs dependencies:
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
    report_progress("Detecting package managers...", 10);

    // Detect ALL available package managers for fallback
    let mut available_managers = Vec::new();

    if Command::new("apt-get").arg("--version").output().is_ok() {
        available_managers.push(("apt-get", vec!["install", "-y"], "libimage-exiftool-perl"));
    }
    if Command::new("dnf").arg("--version").output().is_ok() {
        available_managers.push(("dnf", vec!["install", "-y"], "perl-Image-ExifTool"));
    }
    if Command::new("yum").arg("--version").output().is_ok() {
        available_managers.push(("yum", vec!["install", "-y"], "perl-Image-ExifTool"));
    }
    if Command::new("pacman").arg("--version").output().is_ok() {
        available_managers.push(("pacman", vec!["-S", "--noconfirm"], "perl-image-exiftool"));
    }
    if Command::new("snap").arg("version").output().is_ok() {
        available_managers.push(("snap", vec!["install"], "exiftool"));
    }

    if available_managers.is_empty() {
        return Err("No supported package manager found (apt-get, dnf, yum, pacman, or snap required)".to_string());
    }

    println!("Found {} package manager(s): {}",
             available_managers.len(),
             available_managers.iter().map(|(name, _, _)| *name).collect::<Vec<_>>().join(", "));

    // Check if running with sudo/root
    let needs_sudo = std::env::var("USER").unwrap_or_default() != "root";

    // Helper function to try installing with all available package managers
    let try_install_package = |display_name: &str, packages: &[(&str, &str)]| -> bool {
        println!("Installing {}...", display_name);

        // Try each available package manager
        for (pkg_manager, install_cmd, _) in &available_managers {
            // Find package name for this manager
            let package = packages.iter()
                .find(|(manager, _)| manager == pkg_manager)
                .map(|(_, pkg)| *pkg);

            if let Some(package) = package {
                println!("Trying {} with {}...", display_name, pkg_manager);

                let result = if *pkg_manager == "snap" {
                    // snap doesn't need sudo prefix in command
                    let mut cmd = Command::new("sudo");
                    cmd.arg("snap");
                    for arg in install_cmd {
                        cmd.arg(arg);
                    }
                    cmd.arg(package).output()
                } else if needs_sudo {
                    let mut cmd = Command::new("sudo");
                    cmd.arg(pkg_manager);
                    for arg in install_cmd {
                        cmd.arg(arg);
                    }
                    cmd.arg(package).output()
                } else {
                    let mut cmd = Command::new(pkg_manager);
                    for arg in install_cmd {
                        cmd.arg(arg);
                    }
                    cmd.arg(package).output()
                };

                match result {
                    Ok(output) if output.status.success() => {
                        println!("{} installed successfully with {}", display_name, pkg_manager);
                        return true;
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        println!("{} failed with {}: {}", display_name, pkg_manager, stderr);

                        // Check for DNS/network errors
                        if stderr.contains("Could not resolve") ||
                           stderr.contains("Temporary failure in name resolution") ||
                           stderr.contains("Name or service not known") {
                            println!("Detected DNS error, trying DNS fallback...");

                            if super::try_with_public_dns().is_ok() {
                                println!("Retrying {} with public DNS...", display_name);

                                let retry = if *pkg_manager == "snap" {
                                    let mut cmd = Command::new("sudo");
                                    cmd.arg("snap");
                                    for arg in install_cmd {
                                        cmd.arg(arg);
                                    }
                                    cmd.arg(package).output()
                                } else if needs_sudo {
                                    let mut cmd = Command::new("sudo");
                                    cmd.arg(pkg_manager);
                                    for arg in install_cmd {
                                        cmd.arg(arg);
                                    }
                                    cmd.arg(package).output()
                                } else {
                                    let mut cmd = Command::new(pkg_manager);
                                    for arg in install_cmd {
                                        cmd.arg(arg);
                                    }
                                    cmd.arg(package).output()
                                };

                                super::restore_dns();

                                if let Ok(retry_output) = retry {
                                    if retry_output.status.success() {
                                        println!("{} installed successfully with DNS fallback", display_name);
                                        return true;
                                    }
                                }
                            }
                        }

                        // Try next package manager
                        continue;
                    }
                    Err(e) => {
                        println!("Failed to execute {} command: {}", pkg_manager, e);
                        continue;
                    }
                }
            }
        }

        false
    };

    // Install exiftool (required)
    report_progress("Installing exiftool (required)...", 30);

    let exiftool_packages = vec![
        ("apt-get", "libimage-exiftool-perl"),
        ("dnf", "perl-Image-ExifTool"),
        ("yum", "perl-Image-ExifTool"),
        ("pacman", "perl-image-exiftool"),
        ("snap", "exiftool"),
    ];

    if !try_install_package("exiftool", &exiftool_packages) {
        super::restore_dns();
        return Err("Failed to install exiftool with all available package managers".to_string());
    }

    // Install tesseract (optional)
    report_progress("Installing tesseract (optional OCR support)...", 50);

    let tesseract_packages = vec![
        ("apt-get", "tesseract-ocr"),
        ("dnf", "tesseract"),
        ("yum", "tesseract"),
        ("pacman", "tesseract"),
        ("snap", "tesseract"),
    ];

    if !try_install_package("tesseract", &tesseract_packages) {
        println!("WARNING: Tesseract (OCR) installation failed (optional)");
        println!("  OCR support will be disabled");
        println!("  Install manually with your package manager");
    }

    // Install ffmpeg (optional)
    report_progress("Installing ffmpeg (optional video support)...", 70);

    let ffmpeg_packages = vec![
        ("apt-get", "ffmpeg"),
        ("dnf", "ffmpeg"),
        ("yum", "ffmpeg"),
        ("pacman", "ffmpeg"),
        ("snap", "ffmpeg"),
    ];

    if !try_install_package("ffmpeg", &ffmpeg_packages) {
        println!("WARNING: FFmpeg installation failed (optional)");
        println!("  Video frame extraction will be disabled");
        println!("  Install manually with your package manager");
    }

    // Install imagemagick (optional)
    report_progress("Installing imagemagick (optional HEIC support)...", 90);

    let imagemagick_packages = vec![
        ("apt-get", "imagemagick"),
        ("dnf", "ImageMagick"),
        ("yum", "ImageMagick"),
        ("pacman", "imagemagick"),
        ("snap", "imagemagick"),
    ];

    if !try_install_package("imagemagick", &imagemagick_packages) {
        println!("WARNING: ImageMagick installation failed (optional)");
        println!("  HEIC/HEIF image support will be disabled");
        println!("  Install manually with your package manager");
    }

    report_progress("Linux dependencies installed", 100);
    Ok(())
}
