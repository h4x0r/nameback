use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Internal modules (private)
mod code_docstring;
mod deps;
mod detector;
mod dir_context;
mod extractor;
mod format_handlers;
mod generator;
mod image_ocr;
mod key_phrases;
mod location_timestamp;
mod pdf_content;
mod renamer;
mod scorer;
mod series_detector;
mod stem_analyzer;
mod text_content;
mod video_ocr;

// Re-export public types
pub use detector::FileCategory;

/// Configuration options for the rename engine
#[derive(Debug, Clone)]
pub struct RenameConfig {
    /// Skip hidden files and directories (starting with .)
    pub skip_hidden: bool,
    /// Include GPS location in filenames (for photos/videos)
    pub include_location: bool,
    /// Include formatted timestamp in filenames
    pub include_timestamp: bool,
    /// Use multi-frame video analysis (slower but better OCR)
    pub multiframe_video: bool,
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self {
            skip_hidden: false,
            include_location: false,
            include_timestamp: false,
            multiframe_video: true, // Multi-frame video analysis is now the default
        }
    }
}

/// Result of analyzing a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    /// Original file path
    pub original_path: PathBuf,
    /// Original filename
    pub original_name: String,
    /// Proposed new filename (None if no suitable name found)
    pub proposed_name: Option<String>,
    /// File category detected
    pub file_category: FileCategory,
}

/// Result of a rename operation
#[derive(Debug, Clone)]
pub struct RenameResult {
    /// Original file path
    pub original_path: PathBuf,
    /// New filename applied
    pub new_name: String,
    /// Whether the rename was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Main rename engine that handles file analysis and renaming
pub struct RenameEngine {
    config: RenameConfig,
}

impl RenameEngine {
    /// Create a new rename engine with the given configuration
    pub fn new(config: RenameConfig) -> Self {
        Self { config }
    }

    /// Create a rename engine with default configuration
    pub fn default() -> Self {
        Self::new(RenameConfig::default())
    }

    /// Analyze all files in a directory and return proposed renames
    /// This does not perform any actual renaming - use for preview
    pub fn analyze_directory(&self, directory: &Path) -> Result<Vec<FileAnalysis>> {
        let mut analyses = Vec::new();

        // Scan files
        let files = self.scan_files(directory)?;

        // Pre-populate existing names
        let mut existing_names = HashSet::new();
        for file_path in &files {
            if let Some(filename) = file_path.file_name() {
                if let Some(name) = filename.to_str() {
                    existing_names.insert(name.to_string());
                }
            }
        }

        // Analyze each file
        for file_path in files {
            match self.analyze_file(&file_path, &mut existing_names) {
                Ok(analysis) => analyses.push(analysis),
                Err(e) => {
                    log::warn!("Failed to analyze {}: {}", file_path.display(), e);
                    // Still add to results but with no proposed name
                    if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                        analyses.push(FileAnalysis {
                            original_path: file_path.clone(),
                            original_name: name.to_string(),
                            proposed_name: None,
                            file_category: FileCategory::Unknown,
                        });
                    }
                }
            }
        }

        Ok(analyses)
    }

    /// Rename files based on analysis results
    /// Only renames files where analysis.proposed_name is Some()
    pub fn rename_files(&self, analyses: &[FileAnalysis], dry_run: bool) -> Vec<RenameResult> {
        let mut results = Vec::new();

        for analysis in analyses {
            if let Some(new_name) = &analysis.proposed_name {
                match renamer::rename_file(&analysis.original_path, new_name, dry_run) {
                    Ok(_) => {
                        results.push(RenameResult {
                            original_path: analysis.original_path.clone(),
                            new_name: new_name.clone(),
                            success: true,
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(RenameResult {
                            original_path: analysis.original_path.clone(),
                            new_name: new_name.clone(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }
        }

        results
    }

    /// Analyze and rename files in one step (like the original CLI behavior)
    pub fn process_directory(&self, directory: &Path, dry_run: bool) -> Result<Vec<RenameResult>> {
        let analyses = self.analyze_directory(directory)?;
        Ok(self.rename_files(&analyses, dry_run))
    }

    // Private helper methods

    fn scan_files(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();

        for entry in WalkDir::new(directory)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if self.config.skip_hidden {
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
                    log::warn!("Failed to access entry: {}", e);
                }
            }
        }

        Ok(files)
    }

    fn analyze_file(
        &self,
        file_path: &Path,
        existing_names: &mut HashSet<String>,
    ) -> Result<FileAnalysis> {
        // Detect file type
        let file_category = detector::detect_file_type(file_path)?;

        let original_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Skip unknown file types
        if file_category == FileCategory::Unknown {
            return Ok(FileAnalysis {
                original_path: file_path.to_path_buf(),
                original_name,
                proposed_name: None,
                file_category,
            });
        }

        // Extract metadata with configuration
        let metadata = match extractor::extract_metadata(file_path, &self.config) {
            Ok(m) => m,
            Err(_) => {
                return Ok(FileAnalysis {
                    original_path: file_path.to_path_buf(),
                    original_name,
                    proposed_name: None,
                    file_category,
                });
            }
        };

        // Extract candidate name
        let candidate_name = metadata.extract_name(&file_category, file_path);

        let proposed_name = candidate_name.map(|name| {
            let extension = file_path.extension();
            generator::generate_filename(&name, extension, existing_names)
        });

        Ok(FileAnalysis {
            original_path: file_path.to_path_buf(),
            original_name,
            proposed_name,
            file_category,
        })
    }
}

/// Check if all required dependencies are installed
pub fn check_dependencies() -> Result<()> {
    deps::print_dependency_status();
    Ok(())
}

/// Install missing dependencies (interactive)
pub fn install_dependencies() -> Result<()> {
    deps::run_installer().map_err(|e| anyhow::anyhow!(e))
}
