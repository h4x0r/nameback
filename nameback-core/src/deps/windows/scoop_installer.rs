//! Scoop package manager installer for Windows
//!
//! Handles the installation of the Scoop package manager and provides
//! the path to the scoop.cmd executable for package installation.

use std::process::Command;
use crate::deps::{constants, msi_progress};

/// Progress reporting callback type
type ProgressCallback = Box<dyn Fn(&str, u8) + Send + Sync>;

/// Ensures Scoop package manager is installed on Windows
///
/// Checks if Scoop is already installed. If not, downloads and installs it.
/// Handles DNS fallback for network connectivity issues.
///
/// # Arguments
/// * `report_progress` - Callback for progress reporting
///
/// # Returns
/// * `Ok(String)` - Path to scoop.cmd executable
/// * `Err(String)` - Error message if installation fails
pub fn ensure_scoop_installed(
    report_progress: impl Fn(&str, u8) + Send + Sync + 'static
) -> Result<String, String> {
    report_progress("Checking Scoop installation...", 10);

    // Get USERPROFILE path for Scoop installation location
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "USERPROFILE environment variable not set".to_string())?;

    println!("=== DEBUG: Environment Information ===");
    println!("USERPROFILE: {}", user_profile);
    println!("COMSPEC: {}", std::env::var("COMSPEC").unwrap_or_else(|_| "<not set>".to_string()));
    println!("PATH: {}", std::env::var("PATH").unwrap_or_else(|_| "<not set>".to_string()));

    // Check if Scoop is installed
    println!("=== DEBUG: Checking Scoop Installation ===");
    println!("Command: powershell -NoProfile -Command \"Get-Command scoop -ErrorAction SilentlyContinue\"");

    let scoop_check = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Get-Command scoop -ErrorAction SilentlyContinue")
        .output();

    match &scoop_check {
        Ok(output) => {
            println!("Scoop check exit code: {:?}", output.status.code());
            println!("Scoop check stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Scoop check stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to run scoop check command: {}", e);
        }
    }

    let scoop_installed = scoop_check
        .map(|o| o.status.success())
        .unwrap_or(false);

    println!("Scoop installed: {}", scoop_installed);

    if !scoop_installed {
        report_progress("Installing Scoop package manager (using admin rights)...", 20);

        // Use a temp file approach to avoid the Security module issue
        // Download installer to temp, execute with -RunAsAdmin (we have UAC already)
        let temp_dir = std::env::var("TEMP").unwrap_or_else(|_| format!("{}\\AppData\\Local\\Temp", user_profile));
        let installer_path = format!("{}\\install-scoop.ps1", temp_dir);

        println!("=== DEBUG: Installing Scoop ===");
        println!("Temp installer path: {}", installer_path);

        // Download the Scoop installer to a temp file
        let download_cmd = format!(
            "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            constants::SCOOP_INSTALL,
            installer_path
        );

        println!("Download command: powershell -NoProfile -Command \"{}\"", download_cmd);

        let download_result = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(&download_cmd)
            .output()
            .map_err(|e| {
                eprintln!("Failed to execute PowerShell command to download Scoop installer: {}", e);
                format!(
                    "\n╔══════════════════════════════════════════════════════════════════╗\n\
                     ║  POWERSHELL EXECUTION ERROR                                      ║\n\
                     ╚══════════════════════════════════════════════════════════════════╝\n\n\
                     Unable to execute PowerShell to download Scoop installer.\n\n\
                     Error: {}\n\n\
                     This may indicate:\n\
                     • PowerShell is not available or not in PATH\n\
                     • Execution policies are too restrictive\n\
                     • System resource limitations\n\n\
                     Please try installing dependencies manually:\n\
                     • Visit https://scoop.sh for Scoop installation\n\
                     • Or download tools directly from their websites\n", e
                )
            })?;

        println!("Download exit code: {:?}", download_result.status.code());
        println!("Download stdout: {}", String::from_utf8_lossy(&download_result.stdout));
        println!("Download stderr: {}", String::from_utf8_lossy(&download_result.stderr));

        if !download_result.status.success() {
            let stderr = String::from_utf8_lossy(&download_result.stderr);
            eprintln!("Failed to download Scoop installer!");
            eprintln!("  stderr: {}", stderr);

            // Check if it's a DNS/network error
            let is_dns_error = stderr.contains("could not be resolved") ||
                               stderr.contains("unable to resolve") ||
                               stderr.contains("DNS");
            let is_network_error = stderr.contains("Unable to connect") ||
                                   stderr.contains("connection") ||
                                   stderr.contains("network");

            if is_dns_error || is_network_error {
                println!("Detected DNS/network error, attempting DNS fallback to public DNS servers...");
                msi_progress::report_action_data("DNS error detected, trying public DNS servers...");

                // Try switching to public DNS and retrying
                if let Ok(()) = super::try_with_public_dns() {
                    println!("Retrying Scoop installer download with public DNS...");

                    let download_retry = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg(&download_cmd)
                        .output();

                    // Restore DNS regardless of outcome
                    super::restore_dns();

                    match download_retry {
                        Ok(output) if output.status.success() => {
                            println!("Scoop installer downloaded successfully with public DNS!");
                            // Continue with installation - the download succeeded
                        }
                        _ => {
                            let error_msg = format!(
                                "\n╔══════════════════════════════════════════════════════════════════╗\n\
                                 ║  NETWORK CONNECTION ERROR                                        ║\n\
                                 ╚══════════════════════════════════════════════════════════════════╝\n\n\
                                 Unable to download Scoop installer even with public DNS fallback.\n\n\
                                 Possible causes:\n\
                                 • Complete network outage\n\
                                 • Firewall or proxy blocking all connections\n\
                                 • VPN interference\n\n\
                                 Manual installation option:\n\
                                 You can install dependencies manually:\n\
                                 • Visit https://scoop.sh for Scoop installation\n\
                                 • After Scoop is installed, run:\n\
                                   scoop install exiftool tesseract ffmpeg imagemagick\n\n\
                                 Or download dependencies directly:\n\
                                 • ExifTool: https://exiftool.org/\n\
                                 • Tesseract: https://github.com/UB-Mannheim/tesseract/wiki\n\
                                 • FFmpeg: https://ffmpeg.org/download.html\n\
                                 • ImageMagick: https://imagemagick.org/script/download.php\n"
                            );
                            return Err(error_msg);
                        }
                    }
                } else {
                    println!("WARNING: Could not switch to public DNS, continuing with original error...");
                    let error_msg = format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  NETWORK CONNECTION ERROR                                        ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         Unable to download Scoop installer due to network issues.\n\n\
                         Possible causes:\n\
                         • DNS resolution failure (cannot resolve 'get.scoop.sh')\n\
                         • Network connectivity problems\n\
                         • Firewall or proxy blocking the connection\n\
                         • VPN interference\n\n\
                         Troubleshooting steps:\n\
                         1. Check your internet connection\n\
                         2. Try accessing https://get.scoop.sh in a web browser\n\
                         3. Check DNS settings (try 8.8.8.8 or 1.1.1.1)\n\
                         4. Disable VPN temporarily and retry\n\
                         5. Check firewall/antivirus settings\n\n\
                         Manual installation option:\n\
                         You can install dependencies manually:\n\
                         • Visit https://scoop.sh for Scoop installation\n\
                         • After Scoop is installed, run:\n\
                           scoop install exiftool tesseract ffmpeg imagemagick\n\n\
                         Or download dependencies directly:\n\
                         • ExifTool: https://exiftool.org/\n\
                         • Tesseract: https://github.com/UB-Mannheim/tesseract/wiki\n\
                         • FFmpeg: https://ffmpeg.org/download.html\n\
                         • ImageMagick: https://imagemagick.org/script/download.php\n"
                    );
                    return Err(error_msg);
                }
            }

            return Err(format!("Failed to download Scoop installer: {}", stderr));
        }

        // Execute the installer with -RunAsAdmin flag (we already have UAC permission)
        let install_cmd = format!("& '{}' -RunAsAdmin; Remove-Item '{}'", installer_path, installer_path);

        println!("Install command: powershell -NoProfile -ExecutionPolicy Bypass -Command \"{}\"", install_cmd);

        let scoop_install = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&install_cmd)
            .output()
            .map_err(|e| {
                eprintln!("Failed to execute Scoop installer: {}", e);
                format!("Failed to execute Scoop installer: {}", e)
            })?;

        println!("Scoop install exit code: {:?}", scoop_install.status.code());
        println!("Scoop install stdout: {}", String::from_utf8_lossy(&scoop_install.stdout));
        println!("Scoop install stderr: {}", String::from_utf8_lossy(&scoop_install.stderr));

        if !scoop_install.status.success() {
            let stderr = String::from_utf8_lossy(&scoop_install.stderr);
            let stdout = String::from_utf8_lossy(&scoop_install.stdout);
            eprintln!("Scoop installation failed!");
            eprintln!("  stdout: {}", stdout);
            eprintln!("  stderr: {}", stderr);

            // Check for common failure patterns
            let is_network_related = stderr.contains("could not be resolved") ||
                                     stderr.contains("Unable to connect") ||
                                     stderr.contains("network") ||
                                     stderr.contains("connection");

            if is_network_related {
                return Err(format!(
                    "\n╔══════════════════════════════════════════════════════════════════╗\n\
                     ║  SCOOP INSTALLATION FAILED - NETWORK ERROR                       ║\n\
                     ╚══════════════════════════════════════════════════════════════════╝\n\n\
                     The Scoop installer encountered a network error.\n\n\
                     Error details:\n{}\n\n\
                     Please check your network connection and try again.\n\n\
                     Manual installation:\n\
                     1. Fix your network/DNS issues first\n\
                     2. Visit https://scoop.sh and follow installation instructions\n\
                     3. After Scoop is installed, run:\n\
                        scoop install exiftool tesseract ffmpeg imagemagick\n", stderr
                ));
            }

            return Err(format!(
                "\n╔══════════════════════════════════════════════════════════════════╗\n\
                 ║  SCOOP INSTALLATION FAILED                                       ║\n\
                 ╚══════════════════════════════════════════════════════════════════╝\n\n\
                 The Scoop package manager failed to install.\n\n\
                 Error details:\n{}\n\n\
                 Manual installation:\n\
                 1. Visit https://scoop.sh for installation instructions\n\
                 2. Open PowerShell and run:\n\
                    Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser\n\
                    Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression\n\
                 3. After Scoop is installed, run:\n\
                    scoop install exiftool tesseract ffmpeg imagemagick\n", stderr
            ));
        }

        println!("Scoop installed successfully to {}", user_profile);
    }

    // Return the path to scoop.cmd for package installation
    let scoop_cmd = format!("{}\\scoop\\shims\\scoop.cmd", user_profile);
    Ok(scoop_cmd)
}
