//! macOS-specific dependency installation
//!
//! This module handles dependency installation on macOS using:
//! - Homebrew (primary package manager)
//! - MacPorts (fallback package manager)
//! - DNS fallback for network connectivity issues

use super::*;
use std::process::Command;

mod dns_fallback;
mod homebrew;
mod macports;

pub use dns_fallback::{try_with_public_dns, restore_dns};
pub use homebrew::install_dependencies;
