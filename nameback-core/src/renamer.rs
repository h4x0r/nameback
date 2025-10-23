use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;

/// Renames a file, either in dry-run mode (preview only) or actual mode
pub fn rename_file(old_path: &Path, new_filename: &str, dry_run: bool) -> Result<()> {
    let parent = old_path.parent().context("File has no parent directory")?;

    let new_path = parent.join(new_filename);

    // Check if source file exists
    if !old_path.exists() {
        anyhow::bail!("Source file does not exist: {}", old_path.display());
    }

    // Check if destination file already exists (prevent overwrite)
    if new_path.exists() && new_path != old_path {
        anyhow::bail!(
            "Destination file already exists: {}. Skipping to prevent data loss.",
            new_path.display()
        );
    }

    // Check write permissions on parent directory
    if !dry_run {
        let metadata =
            fs::metadata(parent).context("Failed to check parent directory permissions")?;

        if metadata.permissions().readonly() {
            anyhow::bail!("No write permission for directory: {}", parent.display());
        }
    }

    // Perform rename or log dry-run
    if dry_run {
        info!("[DRY RUN] {} -> {}", old_path.display(), new_filename);
    } else {
        fs::rename(old_path, &new_path).context(format!(
            "Failed to rename {} to {}",
            old_path.display(),
            new_path.display()
        ))?;

        info!("Renamed: {} -> {}", old_path.display(), new_filename);
    }

    Ok(())
}

/// Processes a single file: detects type, extracts metadata, generates name, and renames
pub fn process_file(
    file_path: &Path,
    dry_run: bool,
    existing_names: &mut std::collections::HashSet<String>,
) -> Result<()> {
    use crate::detector;
    use crate::extractor;
    use crate::generator;

    // Detect file type
    let file_category =
        detector::detect_file_type(file_path).context("Failed to detect file type")?;

    // Skip unknown file types
    if file_category == detector::FileCategory::Unknown {
        warn!("Skipping unknown file type: {}", file_path.display());
        return Ok(());
    }

    // Extract metadata (using default config for this legacy function)
    let config = crate::RenameConfig::default();
    let metadata = match extractor::extract_metadata(file_path, &config) {
        Ok(m) => m,
        Err(e) => {
            warn!(
                "Failed to extract metadata from {}: {}. Skipping.",
                file_path.display(),
                e
            );
            return Ok(());
        }
    };

    // Extract candidate name from metadata (now with intelligent scoring)
    let candidate_name = match metadata.extract_name(&file_category, file_path) {
        Some(name) => name,
        None => {
            warn!(
                "No suitable metadata found for renaming: {}. Skipping.",
                file_path.display()
            );
            return Ok(());
        }
    };

    // Get original extension
    let extension = file_path.extension();

    // Generate sanitized, unique filename with metadata enhancements
    let new_filename = generator::generate_filename_with_metadata(
        &candidate_name,
        extension,
        existing_names,
        Some(&metadata),
    );

    // Rename the file
    rename_file(file_path, &new_filename, dry_run)?;

    Ok(())
}
