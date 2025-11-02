//! Chocolatey package manager integration

use std::process::Command;
use crate::deps::msi_progress;

/// Ensures Chocolatey is installed, installing it if necessary
///
/// # Returns
/// * `Ok(())` if Chocolatey is available (was already installed or just installed successfully)
/// * `Err(String)` if Chocolatey installation failed
pub fn ensure_chocolatey_installed() -> Result<(), String> {
    // Check if Chocolatey is installed
    // Refresh PATH first in case it was just installed by another process
    let choco_check = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); Get-Command choco -ErrorAction SilentlyContinue")
        .output();

    let choco_installed = choco_check
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !choco_installed {
        println!("Chocolatey not found, installing...");
        msi_progress::report_action_data("Installing Chocolatey package manager...");

        // Install Chocolatey
        let choco_install = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))")
            .output();

        match choco_install {
            Ok(output) if output.status.success() => {
                println!("Chocolatey installed successfully");
                msi_progress::report_action_data("Chocolatey installed");
                Ok(())
            }
            _ => {
                msi_progress::report_action_data("Chocolatey installation failed");
                Err("Failed to install Chocolatey package manager".to_string())
            }
        }
    } else {
        Ok(())
    }
}

/// Installs a package via Chocolatey (assumes Chocolatey is already installed)
///
/// # Arguments
/// * `package_name` - Name of the Chocolatey package (e.g., "exiftool", "tesseract")
///
/// # Returns
/// * `Ok((stdout, stderr))` if installation succeeded
/// * `Err(String)` if installation failed
pub fn install_package_via_chocolatey(package_name: &str) -> Result<(String, String), String> {
    println!("Installing {} via Chocolatey...", package_name);
    msi_progress::report_action_data(&format!("Installing {} via Chocolatey...", package_name));

    let choco_install_cmd = format!(
        "$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); choco install {} -y --no-progress",
        package_name
    );

    let choco_result = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(&choco_install_cmd)
        .output();

    match choco_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            println!("Chocolatey {} stdout: {}", package_name, stdout);
            println!("Chocolatey {} stderr: {}", package_name, stderr);

            if output.status.success() && !stdout.contains("ERROR") {
                println!("{} installed successfully via Chocolatey", package_name);
                msi_progress::report_action_data(&format!("{} installed via Chocolatey", package_name));
                Ok((stdout, stderr))
            } else {
                Err(format!("Chocolatey installation failed for {}", package_name))
            }
        }
        Err(e) => {
            Err(format!("Failed to execute Chocolatey command for {}: {}", package_name, e))
        }
    }
}
