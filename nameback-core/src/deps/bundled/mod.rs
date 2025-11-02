//! Bundled dependency installer fallback
//!
//! This module provides bundled installers as a final fallback when
//! package managers fail. Downloads pre-packaged binaries from GitHub Releases.

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::install_from_bundled;
