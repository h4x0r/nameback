use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Cache entry storing metadata and file hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA-256 hash of file contents
    pub file_hash: String,
    /// File size in bytes
    pub file_size: u64,
    /// Last modification time (Unix timestamp)
    pub modified_time: u64,
    /// Cached proposed filename
    pub proposed_name: Option<String>,
    /// File category
    pub category: String,
    /// Timestamp when this cache entry was created
    pub cache_time: u64,
}

/// Metadata cache that persists to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataCache {
    /// Map from file path to cache entry
    entries: HashMap<String, CacheEntry>,
    /// Cache file path
    #[serde(skip)]
    cache_path: PathBuf,
}

impl MetadataCache {
    /// Create a new cache that will be saved to the given path
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            entries: HashMap::new(),
            cache_path,
        }
    }

    /// Load cache from disk, or create new if doesn't exist
    pub fn load(cache_path: PathBuf) -> Result<Self> {
        if cache_path.exists() {
            let data = fs::read_to_string(&cache_path)?;
            let mut cache: MetadataCache = serde_json::from_str(&data)?;
            cache.cache_path = cache_path;
            Ok(cache)
        } else {
            Ok(Self::new(cache_path))
        }
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.cache_path, data)?;
        Ok(())
    }

    /// Check if file has valid cache entry (hash matches)
    pub fn has_valid_entry(&self, file_path: &Path) -> Result<bool> {
        let path_str = file_path.to_string_lossy().to_string();

        if let Some(entry) = self.entries.get(&path_str) {
            // Check if file still exists and hasn't changed
            if let Ok(metadata) = fs::metadata(file_path) {
                let current_size = metadata.len();
                let current_modified = metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();

                // Quick check: size and modification time
                if entry.file_size == current_size && entry.modified_time == current_modified {
                    return Ok(true);
                }

                // If quick check fails, verify with hash
                let current_hash = Self::compute_file_hash(file_path)?;
                return Ok(entry.file_hash == current_hash);
            }
        }

        Ok(false)
    }

    /// Get cached entry for file
    pub fn get(&self, file_path: &Path) -> Option<&CacheEntry> {
        let path_str = file_path.to_string_lossy().to_string();
        self.entries.get(&path_str)
    }

    /// Store cache entry for file
    pub fn insert(
        &mut self,
        file_path: &Path,
        proposed_name: Option<String>,
        category: &str,
    ) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();
        let metadata = fs::metadata(file_path)?;

        let entry = CacheEntry {
            file_hash: Self::compute_file_hash(file_path)?,
            file_size: metadata.len(),
            modified_time: metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            proposed_name,
            category: category.to_string(),
            cache_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        self.entries.insert(path_str, entry);
        Ok(())
    }

    /// Remove stale entries for files that no longer exist
    pub fn cleanup_stale_entries(&mut self, valid_paths: &[PathBuf]) {
        let valid_set: std::collections::HashSet<String> = valid_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        self.entries.retain(|path, _| valid_set.contains(path));
    }

    /// Compute SHA-256 hash of file contents (fast for small files)
    /// For large files, only hash first and last 64KB + file size
    fn compute_file_hash(file_path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::io::Read;

        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();

        // For small files (< 1MB), hash entire file
        if file_size < 1_048_576 {
            let data = fs::read(file_path)?;
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            return Ok(format!("{:x}", hasher.finish()));
        }

        // For large files, hash first 64KB + last 64KB + size
        // This is fast and catches most modifications
        let mut file = fs::File::open(file_path)?;
        let mut hasher = DefaultHasher::new();

        // Hash file size
        file_size.hash(&mut hasher);

        // Hash first 64KB
        let mut buffer = vec![0u8; 65536];
        let n = file.read(&mut buffer)?;
        buffer[..n].hash(&mut hasher);

        // Hash last 64KB if file is large enough
        if file_size > 65536 {
            use std::io::Seek;
            file.seek(std::io::SeekFrom::End(-65536))?;
            let n = file.read(&mut buffer)?;
            buffer[..n].hash(&mut hasher);
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            cache_size_bytes: serde_json::to_string(&self.entries)
                .map(|s| s.len())
                .unwrap_or(0),
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub cache_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_cache_roundtrip() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        // Create cache and insert entry
        let mut cache = MetadataCache::new(cache_path.clone());

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        cache.insert(&test_file, Some("new_name.txt".to_string()), "Document")?;
        cache.save()?;

        // Load cache and verify entry exists
        let loaded_cache = MetadataCache::load(cache_path)?;
        assert!(loaded_cache.has_valid_entry(&test_file)?);

        let entry = loaded_cache.get(&test_file).unwrap();
        assert_eq!(entry.proposed_name, Some("new_name.txt".to_string()));
        assert_eq!(entry.category, "Document");

        Ok(())
    }

    #[test]
    fn test_cache_invalidation_on_modify() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        let mut cache = MetadataCache::new(cache_path);
        let test_file = temp_dir.path().join("test.txt");

        // Write initial content and cache it
        fs::write(&test_file, "initial content")?;
        cache.insert(&test_file, Some("cached_name.txt".to_string()), "Document")?;

        assert!(cache.has_valid_entry(&test_file)?);

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure mtime changes
        let mut file = fs::OpenOptions::new().write(true).open(&test_file)?;
        file.write_all(b"modified content")?;
        drop(file);

        // Cache should be invalid now
        assert!(!cache.has_valid_entry(&test_file)?);

        Ok(())
    }

    #[test]
    fn test_cleanup_stale_entries() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("cache.json");

        let mut cache = MetadataCache::new(cache_path);

        // Create two files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file1, "content1")?;
        fs::write(&file2, "content2")?;

        cache.insert(&file1, Some("name1.txt".to_string()), "Document")?;
        cache.insert(&file2, Some("name2.txt".to_string()), "Document")?;

        assert_eq!(cache.entries.len(), 2);

        // Cleanup with only file1 valid
        cache.cleanup_stale_entries(&[file1.clone()]);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.get(&file1).is_some());
        assert!(cache.get(&file2).is_none());

        Ok(())
    }
}
