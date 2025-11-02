//! Windows-specific dependency installation
//!
//! This module handles dependency installation on Windows using:
//! - Scoop (primary package manager)
//! - Chocolatey (fallback package manager)
//! - Bundled installers (final fallback)

mod chocolatey;
mod scoop;

pub use chocolatey::{ensure_chocolatey_installed, install_package_via_chocolatey};
pub use scoop::install_via_scoop;
