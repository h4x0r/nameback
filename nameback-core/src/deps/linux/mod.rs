//! Linux-specific dependency installation
//!
//! This module handles dependency installation on Linux using:
//! - apt-get (Debian, Ubuntu, Kali)
//! - dnf (Fedora, RHEL 8+)
//! - yum (RHEL 7, CentOS 7)
//! - pacman (Arch Linux, Manjaro)
//! - snap (Universal package manager)

use super::*;
use std::process::Command;

mod apt;
mod dnf;
mod pacman;

pub use apt::install_via_apt;
pub use dnf::install_via_dnf;
pub use pacman::install_via_pacman;
