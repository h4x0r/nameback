//! Chocolatey package manager integration

use super::*;

/// Ensures Chocolatey is installed, installing it if necessary
///
/// # Returns
/// * `Ok(())` if Chocolatey is available (was already installed or just installed successfully)
/// * `Err(String)` if Chocolatey installation failed
pub fn ensure_chocolatey_installed() -> Result<(), String> {
    // Implementation will be moved from deps.rs
    todo!("Move from deps.rs windows_helpers module")
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
    // Implementation will be moved from deps.rs
    todo!("Move from deps.rs windows_helpers module")
}
