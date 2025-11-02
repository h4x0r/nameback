//! Bundled Windows installer fallback

use std::process::Command;
use crate::deps::constants;

/// Installs a dependency from bundled installer (final fallback for Windows)
///
/// Downloads pre-packaged binaries from GitHub Releases and installs them.
///
/// # Arguments
/// * `dep_name` - Name of the dependency (e.g., "exiftool")
/// * `platform` - Platform identifier (should be "windows")
///
/// # Returns
/// * `Ok(())` if installation succeeded
/// * `Err(String)` if installation failed
pub fn install_from_bundled(dep_name: &str, platform: &str) -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    let asset_name = format!("deps-{}-{}.zip", dep_name, platform);
    let download_url = format!(
        "{}/v{}/{}",
        constants::GITHUB_RELEASES_BASE, version, asset_name
    );

    println!("\n=== BUNDLED INSTALLER FALLBACK ===");
    println!("Downloading {} from GitHub Release...", dep_name);
    println!("URL: {}", download_url);

    // Use reqwest to download (already in dependencies)
    let response = reqwest::blocking::get(&download_url)
        .map_err(|e| format!("Failed to download bundled installer: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub Release asset not found: {} (HTTP {})", asset_name, response.status()));
    }

    // Save to temp directory
    let temp_dir = std::env::temp_dir();
    let installer_path = temp_dir.join(&asset_name);
    let content = response.bytes()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    std::fs::write(&installer_path, &content)
        .map_err(|e| format!("Failed to write installer: {}", e))?;

    println!("Downloaded to: {}", installer_path.display());
    println!("Installing {}...", dep_name);

    // Extract zip and run installer
    let extract_dir = temp_dir.join(format!("{}-extract", dep_name));
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extract dir: {}", e))?;

    // Use PowerShell to extract
    let extract_cmd = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        installer_path.display(),
        extract_dir.display()
    );

    let extract_result = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&extract_cmd)
        .output();

    if let Ok(output) = extract_result {
        if !output.status.success() {
            return Err(format!("Failed to extract: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
        return Err("Failed to run extraction command".to_string());
    }

    // Run the installer based on dependency type
    match dep_name {
        "tesseract" => {
            // Find and run the setup.exe
            let setup_exe = extract_dir.join("tesseract-windows-setup.exe");
            if setup_exe.exists() {
                Command::new(&setup_exe)
                    .arg("/S")  // Silent install
                    .status()
                    .map_err(|e| format!("Failed to run installer: {}", e))?;
            }
        }
        "exiftool" | "ffmpeg" | "imagemagick" => {
            // For portable versions, copy to a standard location
            let install_dir = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| "C:\\Program Files".to_string());
            let target_dir = std::path::PathBuf::from(&install_dir).join("Nameback").join(dep_name);

            std::fs::create_dir_all(&target_dir)
                .map_err(|e| format!("Failed to create install dir: {}", e))?;

            // Copy files
            let copy_cmd = format!(
                "Copy-Item -Path '{}\\*' -Destination '{}' -Recurse -Force",
                extract_dir.display(),
                target_dir.display()
            );

            Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&copy_cmd)
                .output()
                .map_err(|e| format!("Failed to copy files: {}", e))?;

            println!("Installed to: {}", target_dir.display());
            println!("Note: You may need to add this to your PATH manually.");
        }
        _ => return Err(format!("Unknown dependency: {}", dep_name)),
    }

    Ok(())
}
