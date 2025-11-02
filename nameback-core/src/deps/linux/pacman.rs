//! Pacman package manager integration (Arch Linux, Manjaro)

use super::*;

/// Installs dependencies via pacman
///
/// # Arguments
/// * `progress` - Optional progress callback
///
/// # Returns
/// * `Ok(())` if installation succeeded
/// * `Err(String)` if installation failed
pub fn install_via_pacman(progress: &Option<super::ProgressCallback>) -> Result<(), String> {
    // Implementation will be moved from deps.rs
    todo!("Move Linux pacman installation logic from deps.rs")
}
