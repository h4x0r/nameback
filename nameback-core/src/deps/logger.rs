//! Installation logger for dependency installer
//!
//! Provides dual output to both console and a timestamped log file in %TEMP%
//! for troubleshooting installation issues, especially during MSI installs.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Installation logger that writes to both console and file
pub struct InstallLogger {
    log_file: Arc<Mutex<Option<File>>>,
    log_path: PathBuf,
}

impl InstallLogger {
    /// Create a new logger with a timestamped log file
    pub fn new() -> Result<Self, String> {
        let log_path = Self::create_log_path()?;

        // Create or open the log file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to create log file at {:?}: {}", log_path, e))?;

        let logger = Self {
            log_file: Arc::new(Mutex::new(Some(file))),
            log_path: log_path.clone(),
        };

        // Write header
        logger.info(&format!("=== Nameback Dependency Installation Log ==="));
        logger.info(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
        logger.info(&format!("Log file: {:?}", log_path));
        logger.info(&format!("Started: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        logger.info("");

        Ok(logger)
    }

    /// Create the log file path with timestamp
    fn create_log_path() -> Result<PathBuf, String> {
        // Get temp directory
        let temp_dir = std::env::temp_dir();

        // Create filename with timestamp
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("nameback-install-{}.log", timestamp);

        let log_path = temp_dir.join(filename);
        Ok(log_path)
    }

    /// Get the log file path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// Log an info message
    pub fn info(&self, message: &str) {
        self.log_message("INFO", message);
    }

    /// Log a warning message
    pub fn warn(&self, message: &str) {
        self.log_message("WARN", message);
    }

    /// Log an error message
    pub fn error(&self, message: &str) {
        self.log_message("ERROR", message);
    }

    /// Log a debug message
    pub fn debug(&self, message: &str) {
        self.log_message("DEBUG", message);
    }

    /// Internal method to write log messages
    fn log_message(&self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let formatted = if level.is_empty() {
            // No level prefix (for raw output like stdout/stderr)
            format!("[{}] {}", timestamp, message)
        } else {
            format!("[{}] [{}] {}", timestamp, level, message)
        };

        // Write to log file only (console output handled by report_progress)
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = writeln!(file, "{}", formatted);
                let _ = file.flush();
            }
        }
    }

    /// Log command output (stdout)
    pub fn log_stdout(&self, output: &str) {
        for line in output.lines() {
            self.log_message("", &format!("  stdout: {}", line));
        }
    }

    /// Log command output (stderr)
    pub fn log_stderr(&self, output: &str) {
        for line in output.lines() {
            self.log_message("", &format!("  stderr: {}", line));
        }
    }

    /// Clean up old log files (keep only the last N files)
    pub fn cleanup_old_logs(keep_count: usize) -> Result<(), String> {
        let temp_dir = std::env::temp_dir();

        // Find all nameback install log files
        let mut log_files: Vec<PathBuf> = std::fs::read_dir(&temp_dir)
            .map_err(|e| format!("Failed to read temp directory: {}", e))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("nameback-install-") && n.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time (oldest first)
        log_files.sort_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|m| m.modified())
                .ok()
        });

        // Remove oldest files if we have more than keep_count
        if log_files.len() > keep_count {
            let to_remove = log_files.len() - keep_count;
            for path in log_files.iter().take(to_remove) {
                let _ = std::fs::remove_file(path);
            }
        }

        Ok(())
    }

    /// Finalize the log and report location
    pub fn finalize(&self) {
        self.info("");
        self.info(&format!("Finished: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        self.info(&format!("Log file saved to: {:?}", self.log_path));
    }
}

impl Drop for InstallLogger {
    fn drop(&mut self) {
        // Ensure log file is flushed when logger is dropped
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = file.flush();
            }
        }
    }
}
