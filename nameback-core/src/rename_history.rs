use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

/// A single rename operation in the history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameOperation {
    /// Original file path (before rename)
    pub original_path: PathBuf,
    /// New file path (after rename)
    pub new_path: PathBuf,
    /// Timestamp when the rename occurred (Unix timestamp)
    pub timestamp: u64,
    /// Whether this operation has been undone
    pub undone: bool,
}

impl RenameOperation {
    /// Create a new rename operation
    pub fn new(original_path: PathBuf, new_path: PathBuf) -> Self {
        Self {
            original_path,
            new_path,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            undone: false,
        }
    }

    /// Undo this rename operation (rename back to original)
    pub fn undo(&mut self) -> Result<()> {
        if self.undone {
            anyhow::bail!("Operation already undone");
        }

        // Check if the new path still exists
        if !self.new_path.exists() {
            anyhow::bail!(
                "Cannot undo: File {} no longer exists",
                self.new_path.display()
            );
        }

        // Check if the original path is available
        if self.original_path.exists() {
            anyhow::bail!(
                "Cannot undo: Original path {} is occupied",
                self.original_path.display()
            );
        }

        // Perform the undo (rename back to original)
        fs::rename(&self.new_path, &self.original_path)?;
        self.undone = true;

        log::info!(
            "Undone rename: {} -> {}",
            self.new_path.display(),
            self.original_path.display()
        );

        Ok(())
    }
}

/// Rename history tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameHistory {
    /// Maximum number of operations to keep in history
    max_history: usize,
    /// History of rename operations (newest first)
    operations: VecDeque<RenameOperation>,
    /// Path where history is persisted
    #[serde(skip)]
    history_path: PathBuf,
}

impl RenameHistory {
    /// Create a new history tracker
    pub fn new(history_path: PathBuf, max_history: usize) -> Self {
        Self {
            max_history,
            operations: VecDeque::new(),
            history_path,
        }
    }

    /// Load history from disk, or create new if doesn't exist
    pub fn load(history_path: PathBuf, max_history: usize) -> Result<Self> {
        if history_path.exists() {
            let data = fs::read_to_string(&history_path)?;
            let mut history: RenameHistory = serde_json::from_str(&data)?;
            history.history_path = history_path;
            history.max_history = max_history;
            Ok(history)
        } else {
            Ok(Self::new(history_path, max_history))
        }
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(self)?;
        fs::write(&self.history_path, data)?;
        Ok(())
    }

    /// Add a rename operation to the history
    pub fn add(&mut self, operation: RenameOperation) {
        // Add to front (newest first)
        self.operations.push_front(operation);

        // Trim to max size
        while self.operations.len() > self.max_history {
            self.operations.pop_back();
        }
    }

    /// Get all operations (newest first)
    pub fn operations(&self) -> &VecDeque<RenameOperation> {
        &self.operations
    }

    /// Get the most recent operation that can be undone
    pub fn last_undoable(&self) -> Option<&RenameOperation> {
        self.operations.iter().find(|op| !op.undone)
    }

    /// Undo the most recent operation
    pub fn undo_last(&mut self) -> Result<()> {
        let last_idx = self
            .operations
            .iter()
            .position(|op| !op.undone)
            .ok_or_else(|| anyhow::anyhow!("No operations to undo"))?;

        self.operations[last_idx].undo()?;
        Ok(())
    }

    /// Undo a specific operation by index
    pub fn undo_at(&mut self, index: usize) -> Result<()> {
        if index >= self.operations.len() {
            anyhow::bail!("Invalid operation index");
        }

        self.operations[index].undo()?;
        Ok(())
    }

    /// Get count of undoable operations
    pub fn undoable_count(&self) -> usize {
        self.operations.iter().filter(|op| !op.undone).count()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Get history statistics
    pub fn stats(&self) -> HistoryStats {
        HistoryStats {
            total_operations: self.operations.len(),
            undoable_operations: self.undoable_count(),
            max_history: self.max_history,
        }
    }
}

/// History statistics
#[derive(Debug)]
pub struct HistoryStats {
    pub total_operations: usize,
    pub undoable_operations: usize,
    pub max_history: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rename_operation() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create a test file
        let original = temp_dir.path().join("original.txt");
        fs::write(&original, "test content")?;

        let new = temp_dir.path().join("renamed.txt");

        // Perform rename
        fs::rename(&original, &new)?;

        // Create operation and undo it
        let mut op = RenameOperation::new(original.clone(), new.clone());
        assert!(!op.undone);

        op.undo()?;

        assert!(op.undone);
        assert!(original.exists());
        assert!(!new.exists());

        Ok(())
    }

    #[test]
    fn test_history_add_and_undo() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let history_path = temp_dir.path().join("history.json");

        let mut history = RenameHistory::new(history_path.clone(), 10);

        // Create test files
        let file1_orig = temp_dir.path().join("file1.txt");
        let file1_new = temp_dir.path().join("file1_renamed.txt");
        fs::write(&file1_orig, "content1")?;
        fs::rename(&file1_orig, &file1_new)?;

        let file2_orig = temp_dir.path().join("file2.txt");
        let file2_new = temp_dir.path().join("file2_renamed.txt");
        fs::write(&file2_orig, "content2")?;
        fs::rename(&file2_orig, &file2_new)?;

        // Add operations
        history.add(RenameOperation::new(file1_orig.clone(), file1_new.clone()));
        history.add(RenameOperation::new(file2_orig.clone(), file2_new.clone()));

        assert_eq!(history.operations.len(), 2);
        assert_eq!(history.undoable_count(), 2);

        // Undo last (file2)
        history.undo_last()?;
        assert!(file2_orig.exists());
        assert!(!file2_new.exists());
        assert_eq!(history.undoable_count(), 1);

        // Undo last (file1)
        history.undo_last()?;
        assert!(file1_orig.exists());
        assert!(!file1_new.exists());
        assert_eq!(history.undoable_count(), 0);

        Ok(())
    }

    #[test]
    fn test_history_persistence() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let history_path = temp_dir.path().join("history.json");

        // Create history and add operation
        let mut history = RenameHistory::new(history_path.clone(), 10);

        let file_orig = temp_dir.path().join("file.txt");
        let file_new = temp_dir.path().join("renamed.txt");

        history.add(RenameOperation::new(file_orig.clone(), file_new.clone()));
        history.save()?;

        // Load history and verify
        let loaded = RenameHistory::load(history_path, 10)?;
        assert_eq!(loaded.operations.len(), 1);
        assert_eq!(loaded.operations[0].original_path, file_orig);
        assert_eq!(loaded.operations[0].new_path, file_new);

        Ok(())
    }

    #[test]
    fn test_history_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.json");

        let mut history = RenameHistory::new(history_path, 3);

        // Add more operations than max_history
        for i in 0..5 {
            let op = RenameOperation::new(
                PathBuf::from(format!("file{}.txt", i)),
                PathBuf::from(format!("renamed{}.txt", i)),
            );
            history.add(op);
        }

        // Should only keep the 3 most recent
        assert_eq!(history.operations.len(), 3);

        // Newest should be at front
        assert!(history.operations[0]
            .original_path
            .to_str()
            .unwrap()
            .contains("file4"));
    }
}
