//! Windows-specific dependency installation
//!
//! This module handles dependency installation on Windows using:
//! - Scoop (primary package manager)
//! - Chocolatey (fallback package manager)
//! - Bundled installers (final fallback)

use super::*;
use std::process::Command;

mod chocolatey;
mod scoop;

pub use chocolatey::{ensure_chocolatey_installed, install_package_via_chocolatey};
pub use scoop::install_via_scoop;
