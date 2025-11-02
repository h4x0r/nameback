//! macOS-specific dependency installation
//!
//! This module handles dependency installation on macOS using:
//! - Homebrew (primary package manager)
//! - MacPorts (fallback package manager)

use super::*;
use std::process::Command;

mod homebrew;
mod macports;

pub use homebrew::install_via_homebrew;
pub use macports::install_via_macports;
