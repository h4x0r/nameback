//! Linux-specific dependency installation
//!
//! This module handles dependency installation on Linux using:
//! - apt-get (Debian, Ubuntu, Kali)
//! - dnf (Fedora, RHEL 8+)
//! - yum (RHEL 7, CentOS 7)
//! - pacman (Arch Linux, Manjaro)
//! - snap (Universal package manager)

mod dns_fallback;
mod apt;

pub use dns_fallback::{try_with_public_dns, restore_dns};
pub use apt::install_dependencies;
