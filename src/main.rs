use anyhow::Result;
use log::{info, warn};
use std::path::PathBuf;
use walkdir::WalkDir;

mod cli;
mod detector;
mod extractor;
mod generator;
mod pdf_content;
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
    // Refuse to run as root for security
    #[cfg(unix)]
    {
        if unsafe { libc::geteuid() } == 0 {
            eprintln!("ERROR: nameback refuses to run as root for security reasons.");
            eprintln!("Running as root could accidentally modify system directories.");
            eprintln!("Please run as a regular user.");
            std::process::exit(1);
        }
    }

    let args = cli::parse_args();

    // Initialize logger with appropriate level based on verbose flag
    if std::env::var("RUST_LOG").is_err() {
        if args.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    if args.dry_run {
        info!("Running in DRY-RUN mode - no files will be renamed");
    }

    // Scan files in the target directory
    let files = scan_files(&args.directory, args.skip_hidden)?;

    info!("Processing {} files...", files.len());

    // Pre-populate existing names from all files to avoid duplicates
    let mut existing_names = std::collections::HashSet::new();
    for file_path in &files {
        if let Some(filename) = file_path.file_name() {
            if let Some(name) = filename.to_str() {
                existing_names.insert(name.to_string());
            }
        }
    }

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
