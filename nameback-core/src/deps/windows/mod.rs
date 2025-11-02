//! Windows-specific dependency installation
//!
//! This module handles dependency installation on Windows using:
//! - Scoop (primary package manager)
//! - Chocolatey (fallback package manager)
//! - Bundled installers (final fallback)
//! - DNS fallback for network connectivity issues

mod chocolatey;
mod dns_fallback;
mod scoop;
mod scoop_installer;

pub use chocolatey::{ensure_chocolatey_installed, install_package_via_chocolatey};
pub use dns_fallback::{try_with_public_dns, restore_dns};
pub use scoop::install_via_scoop;
pub use scoop_installer::ensure_scoop_installed;
