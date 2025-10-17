use clap::Parser;
use std::path::PathBuf;

/// A utility to rename files based on their metadata
#[derive(Parser, Debug)]
#[command(name = "nameback")]
#[command(author = "4n6h4x0r")]
#[command(version = "0.2.0")]
#[command(about = "Renames files based on metadata from exiftool", long_about = None)]
pub struct Args {
    /// Directory to scan for files
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<PathBuf>,

    /// Run in dry-run mode (preview changes without renaming)
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Skip hidden files and directories
    #[arg(short = 's', long = "skip-hidden")]
    pub skip_hidden: bool,

    /// Verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Check and install missing dependencies
    #[arg(long = "install-deps")]
    pub install_deps: bool,

    /// Check dependency status without installing
    #[arg(long = "check-deps")]
    pub check_deps: bool,
}

/// Parses command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}
