//! Scoop package manager integration for Windows dependency installation
//!
//! Installs required and optional dependencies via Scoop with Chocolatey/bundled fallbacks.

use std::process::Command;
use crate::deps::msi_progress;

/// Progress reporting callback type
type ProgressCallback = Box<dyn Fn(&str, u8) + Send + Sync>;

/// Installs all dependencies via Scoop with fallback options
///
/// Installs dependencies in order:
/// 1. 7zip (required - needed by Scoop for extraction)
/// 2. exiftool (required - core metadata extraction)
/// 3. tesseract (optional - OCR support)
/// 4. ffmpeg (optional - video frame extraction)
/// 5. imagemagick (optional - HEIC/HEIF support)
///
/// # Arguments
/// * `scoop_cmd` - Path to scoop.cmd executable
/// * `report_progress` - Progress callback for UI updates
///
/// # Returns
/// * `Ok(())` if required dependencies installed successfully
/// * `Err(String)` if required dependencies failed
pub fn install_dependencies_via_scoop(
    scoop_cmd: &str,
    report_progress: impl Fn(&str, u8) + Send + Sync + 'static
) -> Result<(), String> {
    // DNS fallback is now handled before Scoop installation in deps.rs
    // Use full path to cmd.exe
    let cmd_exe = std::env::var("COMSPEC").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".to_string());

    // Install 7zip first - required for extracting other packages
    report_progress("Installing 7zip (required for extracting packages)...", 30);
    msi_progress::report_action_data("Downloading and installing 7zip...");
    println!("=== DEBUG: Installing 7zip ===");
    println!("Full command: {} /c \"{}\" install 7zip", cmd_exe, scoop_cmd);

    let seven_zip_result = Command::new(&cmd_exe)
        .arg("/c")
        .arg(scoop_cmd)
        .arg("install")
        .arg("7zip")
        .output()
        .map_err(|e| {
            msi_progress::report_action_data("ERROR: Failed to execute 7zip install command");
            eprintln!("Failed to execute 7zip install command: {}", e);
            format!("Failed to run scoop install 7zip: {}", e)
        })?;

    println!("7zip install exit code: {:?}", seven_zip_result.status.code());
    println!("7zip install stdout: {}", String::from_utf8_lossy(&seven_zip_result.stdout));
    println!("7zip install stderr: {}", String::from_utf8_lossy(&seven_zip_result.stderr));

    let stderr = String::from_utf8_lossy(&seven_zip_result.stderr);
    let stdout = String::from_utf8_lossy(&seven_zip_result.stdout);

    // Check for actual installation failures (not shimming warnings)
    // On ARM64 Windows, shimming may fail but 7z.exe is installed successfully
    let has_critical_error = !seven_zip_result.status.success() ||
                              stdout.contains("ERROR") ||
                              stdout.contains("Failed to download") ||
                              stdout.contains("Failed to install") ||
                              stdout.contains("Unable to extract");

    // Shimming warnings are non-critical - 7z.exe is still installed
    let is_shimming_warning = stderr.contains("Can't shim") ||
                              stderr.contains("File doesn't exist") ||
                              stderr.contains("Get-Command");

    if has_critical_error && !is_shimming_warning {
        msi_progress::report_action_data("ERROR: 7zip installation failed");
        eprintln!("7zip installation failed!");
        eprintln!("  stdout: {}", stdout);
        eprintln!("  stderr: {}", stderr);
        return Err(format!(
            "\n╔══════════════════════════════════════════════════════════════════╗\n\
             ║  7ZIP INSTALLATION FAILED                                        ║\n\
             ╚══════════════════════════════════════════════════════════════════╝\n\n\
             7zip is required to extract other packages.\n\n\
             Error details:\n{}\n\n\
             Please try installing manually:\n\
             • Run: scoop install 7zip\n", stderr
        ));
    } else if is_shimming_warning {
        println!("Note: 7zip shimming warnings detected (non-critical on ARM64)");
        println!("7z.exe is installed and available at the full path");
    }

    msi_progress::report_action_data("7zip installed successfully");
    println!("7zip installed successfully");

    // Install exiftool (required)
    report_progress("Installing exiftool (required)...", 45);
    msi_progress::report_action_data("Downloading and installing exiftool...");
    println!("=== DEBUG: Installing exiftool ===");
    println!("cmd.exe location: {}", cmd_exe);
    println!("scoop.cmd location: {}", scoop_cmd);
    println!("Full command: {} /c \"{}\" install exiftool", cmd_exe, scoop_cmd);

    let exiftool_result = Command::new(&cmd_exe)
        .arg("/c")
        .arg(scoop_cmd)
        .arg("install")
        .arg("exiftool")
        .output()
        .map_err(|e| {
            msi_progress::report_action_data("ERROR: Failed to execute exiftool install command");
            eprintln!("Failed to execute exiftool install command: {}", e);
            format!("Failed to run scoop install exiftool: {}", e)
        })?;

    println!("exiftool install exit code: {:?}", exiftool_result.status.code());
    println!("exiftool install stdout: {}", String::from_utf8_lossy(&exiftool_result.stdout));
    println!("exiftool install stderr: {}", String::from_utf8_lossy(&exiftool_result.stderr));

    let stdout = String::from_utf8_lossy(&exiftool_result.stdout);
    let stderr = String::from_utf8_lossy(&exiftool_result.stderr);

    // Scoop reports errors in stdout, not stderr, and still exits with code 0
    // Check for common error patterns including SSL/TLS errors
    let has_error = !exiftool_result.status.success() ||
                    stdout.contains("is not valid") ||
                    stdout.contains("ERROR") ||
                    stdout.contains("Failed to") ||
                    stdout.contains("Authentication failed") ||
                    stdout.contains("Unable to connect") ||
                    stdout.contains("could not be resolved") ||
                    stdout.contains("SSL connection") ||
                    stdout.contains("The SSL") ||
                    stdout.contains("certificate");

    if has_error {
        msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
        eprintln!("exiftool installation via Scoop failed!");
        eprintln!("  stdout: {}", stdout);
        eprintln!("  stderr: {}", stderr);
        eprintln!("Attempting fallback to Chocolatey...");

        // Try Chocolatey as fallback
        println!("=== DEBUG: Installing exiftool via Chocolatey (fallback) ===");

        // Ensure Chocolatey is installed
        if let Err(_) = super::ensure_chocolatey_installed() {
            msi_progress::report_action_data("Chocolatey failed, trying bundled installer...");
            println!("Chocolatey installation also failed. Attempting bundled installer fallback...");

            // Try bundled installer as final fallback
            match super::super::bundled::install_from_bundled("exiftool", "windows") {
                Ok(()) => {
                    println!("ExifTool installed successfully from bundled installer!");
                    msi_progress::report_action_data("ExifTool installed from bundled fallback");
                    report_progress("ExifTool installed (bundled fallback)", 25);
                    return Ok(()); // Exit early - dependency installed successfully
                }
                Err(bundle_err) => {
                    msi_progress::report_action_data("ERROR: All installation methods failed");
                    return Err(format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         ExifTool is required for Nameback to function.\n\n\
                         All installation methods failed:\n\
                         • Scoop: {}\n\
                         • Chocolatey: Failed to install Chocolatey\n\
                         • Bundled installer: {}\n\n\
                         Please install manually:\n\
                         • Download from: https://exiftool.org/\n\
                         • Or run: choco install exiftool -y\n", stdout, bundle_err
                    ));
                }
            }
        }

        // Try installing exiftool via Chocolatey
        match super::install_package_via_chocolatey("exiftool") {
            Ok((_choco_stdout, _choco_stderr)) => {
                // Installation succeeded
            }
            Err(_) => {
                msi_progress::report_action_data("ERROR: Chocolatey installation also failed");
                return Err(format!(
                    "\n╔══════════════════════════════════════════════════════════════════╗\n\
                     ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                     ╚══════════════════════════════════════════════════════════════════╝\n\n\
                     ExifTool is required for Nameback to function.\n\n\
                     Both Scoop and Chocolatey failed to install exiftool.\n\n\
                     Scoop error:\n{}\n\n\
                     Please install manually:\n\
                     • Download from: https://exiftool.org/\n", stdout
                ));
            }
        }
    }

    msi_progress::report_action_data("exiftool installed successfully");
    println!("exiftool installed successfully");

    // Install optional dependencies with graceful failure
    install_optional_dependency(&cmd_exe, scoop_cmd, "tesseract", "OCR support", &report_progress, 60);
    install_optional_dependency(&cmd_exe, scoop_cmd, "ffmpeg", "video frame extraction", &report_progress, 80);
    install_optional_dependency(&cmd_exe, scoop_cmd, "imagemagick", "HEIC/HEIF image support", &report_progress, 90);

    report_progress("Windows dependencies installed", 100);
    msi_progress::report_action_data("Dependency installation complete");

    Ok(())
}

/// Installs an optional dependency with Chocolatey fallback
///
/// Does not fail if installation is unsuccessful - only prints warnings.
fn install_optional_dependency(
    cmd_exe: &str,
    scoop_cmd: &str,
    package_name: &str,
    description: &str,
    report_progress: &impl Fn(&str, u8),
    progress_pct: u8
) {
    report_progress(&format!("Installing {} (optional {})...", package_name, description), progress_pct);
    msi_progress::report_action_data(&format!("Downloading and installing {} (optional)...", package_name));
    println!("=== DEBUG: Installing {} (optional) ===", package_name);
    println!("Full command: {} /c \"{}\" install {}", cmd_exe, scoop_cmd, package_name);

    let result = Command::new(cmd_exe)
        .arg("/c")
        .arg(scoop_cmd)
        .arg("install")
        .arg(package_name)
        .output();

    match result {
        Ok(output) => {
            println!("{} install exit code: {:?}", package_name, output.status.code());
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{} install stdout: {}", package_name, stdout);
            println!("{} install stderr: {}", package_name, stderr);

            // Check for errors in stdout (Scoop reports errors there)
            let has_error = !output.status.success() ||
                            stdout.contains("Failed to extract") ||
                            stdout.contains("is not valid") ||
                            stdout.contains("ERROR") ||
                            stdout.contains("could not be resolved");

            if has_error {
                msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
                println!("WARNING: {} installation via Scoop failed!", package_name);
                println!("  stdout: {}", stdout);
                println!("  stderr: {}", stderr);
                println!("Attempting fallback to Chocolatey...");

                // Try Chocolatey as fallback
                println!("=== DEBUG: Installing {} via Chocolatey (fallback) ===", package_name);

                // Ensure Chocolatey is installed
                if let Err(_) = super::ensure_chocolatey_installed() {
                    msi_progress::report_action_data(&format!("WARNING: Both Scoop and Chocolatey failed ({} is optional)", package_name));
                    println!("WARNING: Both Scoop and Chocolatey installation attempts failed for {}", package_name);
                    println!("  {} is optional - only needed for {}", package_name, description);
                    println!("  You can install it manually later: choco install {} -y", package_name);
                } else {
                    // Try installing via Chocolatey
                    match super::install_package_via_chocolatey(package_name) {
                        Ok(_) => {
                            msi_progress::report_action_data(&format!("{} installed successfully", package_name));
                            println!("{} installed successfully via Chocolatey", package_name);
                        }
                        Err(_) => {
                            msi_progress::report_action_data(&format!("WARNING: {} installation failed (optional)", package_name));
                            println!("WARNING: Chocolatey installation also failed for {}", package_name);
                            println!("  {} is optional - only needed for {}", package_name, description);
                        }
                    }
                }
            } else {
                msi_progress::report_action_data(&format!("{} installed successfully", package_name));
                println!("{} installed successfully", package_name);
            }
        }
        Err(e) => {
            msi_progress::report_action_data(&format!("WARNING: {} install command failed", package_name));
            println!("WARNING: Failed to execute {} install command: {}", package_name, e);
        }
    }
}
