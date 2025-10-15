use clap::Parser;
use std::path::PathBuf;

/// A utility to rename files based on their metadata
#[derive(Parser, Debug)]
#[command(name = "nameback")]
#[command(author = "4n6h4x0r")]
#[command(version = "0.1.0")]
#[command(about = "Renames files based on metadata from exiftool", long_about = None)]
pub struct Args {
    /// Directory to scan for files
    #[arg(value_name = "DIRECTORY")]
    pub directory: PathBuf,

    /// Run in dry-run mode (preview changes without renaming)
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Skip hidden files and directories
    #[arg(short = 's', long = "skip-hidden")]
    pub skip_hidden: bool,

    /// Verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}

/// Parses command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}
