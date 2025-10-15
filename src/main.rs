use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;
use walkdir::WalkDir;

mod cli;
mod detector;
mod extractor;
mod generator;
mod renamer;

/// Scans a directory recursively and returns paths to all files
pub fn scan_files(directory: &PathBuf, skip_hidden: bool) -> Result<Vec<PathBuf>> {
    info!("Scanning directory: {}", directory.display());

    let mut files = Vec::new();

    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if skip_hidden {
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            } else {
                true
            }
        })
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_path_buf());
                }
            }
            Err(e) => {
                warn!("Failed to access entry: {}", e);
            }
        }
    }

    info!("Found {} files", files.len());
    Ok(files)
}

fn main() -> Result<()> {
    // Initialize logger with appropriate level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args = cli::parse_args();

    if args.dry_run {
        info!("Running in DRY-RUN mode - no files will be renamed");
    }

    // Scan files in the target directory
    let files = scan_files(&args.directory, args.skip_hidden)?;

    info!("Processing {} files...", files.len());

    // Track existing names to avoid duplicates
    let mut existing_names = std::collections::HashSet::new();

    // Process each file
    for file_path in files {
        match renamer::process_file(&file_path, args.dry_run, &mut existing_names) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to process {}: {}", file_path.display(), e);
            }
        }
    }

    info!("Processing complete!");

    Ok(())
}
