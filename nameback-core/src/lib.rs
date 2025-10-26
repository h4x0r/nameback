use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// Internal modules (private)
mod code_docstring;
mod deps;
mod deps_check;
mod detector;
mod dir_context;
mod extractor;
mod format_handlers;
mod generator;
mod geocoding;
mod image_ocr;
mod key_phrases;
mod location_timestamp;
mod metadata_cache;
mod pdf_content;
mod rename_history;
mod renamer;
mod scorer;
mod series_detector;
mod stem_analyzer;
mod text_content;
mod video_ocr;

// Re-export public types
pub use deps_check::{detect_needed_dependencies, Dependency, DependencyNeeds};
pub use detector::FileCategory;
pub use rename_history::{RenameHistory, RenameOperation};

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
    /// Use geocoding to convert GPS coordinates to city names (defaults to true)
    /// When false, shows coordinates like "47.6N_122.3W" instead of "Seattle_WA"
    pub geocode: bool,
    /// Enable metadata caching to speed up re-analysis
    pub enable_cache: bool,
    /// Cache file path (None = use default location)
    pub cache_path: Option<PathBuf>,
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self {
            skip_hidden: false,
            include_location: true, // Include GPS location by default
            include_timestamp: true, // Include timestamps by default
            multiframe_video: true, // Multi-frame video analysis is now the default
            geocode: true, // Geocoding is enabled by default
            enable_cache: true, // Metadata caching enabled by default
            cache_path: None, // Use default cache location
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

        // Load or create metadata cache
        let cache_path = self.config.cache_path.clone().unwrap_or_else(|| {
            directory.join(".nameback_cache.json")
        });

        let mut cache = if self.config.enable_cache {
            metadata_cache::MetadataCache::load(cache_path.clone()).unwrap_or_else(|_| {
                log::debug!("Failed to load cache, creating new one");
                metadata_cache::MetadataCache::new(cache_path.clone())
            })
        } else {
            metadata_cache::MetadataCache::new(cache_path.clone())
        };

        // Clean up stale cache entries
        if self.config.enable_cache {
            cache.cleanup_stale_entries(&files);
        }

        // Detect file series (e.g., IMG_001.jpg, IMG_002.jpg, etc.)
        let series_list = series_detector::detect_series(&files);
        log::info!("Detected {} file series", series_list.len());

        // Build a map of file paths to their series
        let mut file_series_map = std::collections::HashMap::new();
        for series in &series_list {
            for (file_path, _) in &series.files {
                file_series_map.insert(file_path.clone(), series.clone());
            }
        }

        // Pre-populate existing names
        let mut existing_names = HashSet::new();
        for file_path in &files {
            if let Some(filename) = file_path.file_name() {
                if let Some(name) = filename.to_str() {
                    existing_names.insert(name.to_string());
                }
            }
        }

        // Analyze each file in parallel using rayon
        use rayon::prelude::*;
        use std::sync::Mutex;

        // Wrap existing_names and cache in Mutex for thread-safe access
        let existing_names = Mutex::new(existing_names);
        let cache = Mutex::new(cache);

        // Process files in parallel
        analyses = files
            .par_iter()
            .filter_map(|file_path| {
                // Check cache first if enabled
                if self.config.enable_cache {
                    let cache_guard = cache.lock().unwrap();
                    if let Ok(true) = cache_guard.has_valid_entry(file_path) {
                        if let Some(entry) = cache_guard.get(file_path) {
                            log::debug!("Cache hit for {}", file_path.display());
                            let category = match entry.category.as_str() {
                                "Image" => FileCategory::Image,
                                "Document" => FileCategory::Document,
                                "Audio" => FileCategory::Audio,
                                "Video" => FileCategory::Video,
                                "Email" => FileCategory::Email,
                                "Web" => FileCategory::Web,
                                "Archive" => FileCategory::Archive,
                                "SourceCode" => FileCategory::SourceCode,
                                _ => FileCategory::Unknown,
                            };

                            let original_name = file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            return Some(FileAnalysis {
                                original_path: file_path.clone(),
                                original_name,
                                proposed_name: entry.proposed_name.clone(),
                                file_category: category,
                            });
                        }
                    }
                    drop(cache_guard); // Release lock before analysis
                }

                // Cache miss or caching disabled - analyze the file
                match self.analyze_file_parallel(file_path, &existing_names) {
                    Ok(mut analysis) => {
                        // Check if this file is part of a series
                        if let Some(series) = file_series_map.get(file_path) {
                            // Apply series naming if we have a proposed name
                            if let Some(proposed_name) = &analysis.proposed_name {
                                // Extract just the base name without extension
                                let base_name = if let Some(pos) = proposed_name.rfind('.') {
                                    &proposed_name[..pos]
                                } else {
                                    proposed_name
                                };

                                // Apply series naming pattern
                                if let Some(series_name) = series_detector::apply_series_naming(
                                    series,
                                    file_path,
                                    base_name,
                                ) {
                                    analysis.proposed_name = Some(series_name);
                                }
                            }
                        }

                        // Update cache if enabled
                        if self.config.enable_cache {
                            let mut cache_guard = cache.lock().unwrap();
                            let category_str = match analysis.file_category {
                                FileCategory::Image => "Image",
                                FileCategory::Document => "Document",
                                FileCategory::Audio => "Audio",
                                FileCategory::Video => "Video",
                                FileCategory::Email => "Email",
                                FileCategory::Web => "Web",
                                FileCategory::Archive => "Archive",
                                FileCategory::SourceCode => "SourceCode",
                                FileCategory::Unknown => "Unknown",
                            };

                            if let Err(e) = cache_guard.insert(
                                file_path,
                                analysis.proposed_name.clone(),
                                category_str,
                            ) {
                                log::warn!("Failed to cache entry for {}: {}", file_path.display(), e);
                            }
                        }

                        Some(analysis)
                    },
                    Err(e) => {
                        log::warn!("Failed to analyze {}: {}", file_path.display(), e);
                        // Still add to results but with no proposed name
                        if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                            Some(FileAnalysis {
                                original_path: file_path.clone(),
                                original_name: name.to_string(),
                                proposed_name: None,
                                file_category: FileCategory::Unknown,
                            })
                        } else {
                            None
                        }
                    }
                }
            })
            .collect();

        // Save cache to disk if enabled
        if self.config.enable_cache {
            let cache_guard = cache.lock().unwrap();
            if let Err(e) = cache_guard.save() {
                log::warn!("Failed to save cache: {}", e);
            } else {
                let stats = cache_guard.stats();
                log::info!(
                    "Cached {} entries ({} bytes)",
                    stats.total_entries,
                    stats.cache_size_bytes
                );
            }
        }

        Ok(analyses)
    }

    /// Rename files based on analysis results
    /// Only renames files where analysis.proposed_name is Some()
    pub fn rename_files(&self, analyses: &[FileAnalysis], dry_run: bool) -> Vec<RenameResult> {
        self.rename_files_with_history(analyses, dry_run, None)
    }

    /// Rename files with history tracking
    /// If history is provided, successful renames will be added to the history
    pub fn rename_files_with_history(
        &self,
        analyses: &[FileAnalysis],
        dry_run: bool,
        mut history: Option<&mut RenameHistory>,
    ) -> Vec<RenameResult> {
        let mut results = Vec::new();

        for analysis in analyses {
            if let Some(new_name) = &analysis.proposed_name {
                match renamer::rename_file(&analysis.original_path, new_name, dry_run) {
                    Ok(new_path) => {
                        // Add to history if provided and not dry run
                        if let Some(hist) = history.as_deref_mut() {
                            if !dry_run {
                                let operation = RenameOperation::new(
                                    analysis.original_path.clone(),
                                    new_path.clone(),
                                );
                                hist.add(operation);
                            }
                        }

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
            generator::generate_filename_with_metadata(&name, extension, existing_names, Some(&metadata))
        });

        Ok(FileAnalysis {
            original_path: file_path.to_path_buf(),
            original_name,
            proposed_name,
            file_category,
        })
    }

    /// Parallel version of analyze_file that uses Mutex-protected existing_names
    fn analyze_file_parallel(
        &self,
        file_path: &Path,
        existing_names: &std::sync::Mutex<HashSet<String>>,
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
            // Lock the mutex to access existing_names
            let mut names = existing_names.lock().unwrap();
            generator::generate_filename_with_metadata(&name, extension, &mut names, Some(&metadata))
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

/// Install dependencies with progress callback
pub fn install_dependencies_with_progress(
    progress: Option<deps::ProgressCallback>,
) -> Result<()> {
    deps::run_installer_with_progress(progress).map_err(|e| anyhow::anyhow!(e))
}

/// Re-export progress callback type
pub use deps::ProgressCallback;
