use std::process::Command;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// Platform-specific dependency installation modules
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
mod bundled;

// Constants for external URLs and installation
mod constants {
    /// GitHub Release URLs
    pub const GITHUB_RELEASES_BASE: &str = "https://github.com/h4x0r/nameback/releases/download";

    /// Package manager installation URLs
    #[allow(dead_code)] // Used in error messages and future refactoring
    pub const SCOOP_INSTALL: &str = "https://get.scoop.sh";
    #[allow(dead_code)]
    pub const SCOOP_WEBSITE: &str = "https://scoop.sh";
    #[allow(dead_code)]
    pub const CHOCOLATEY_INSTALL: &str = "https://community.chocolatey.org/install.ps1";
    #[allow(dead_code)]
    pub const HOMEBREW_WEBSITE: &str = "https://brew.sh";
    #[allow(dead_code)]
    pub const MACPORTS_WEBSITE: &str = "https://www.macports.org";

    /// Dependency download URLs
    #[allow(dead_code)]
    pub const EXIFTOOL_WEBSITE: &str = "https://exiftool.org/";
    #[allow(dead_code)]
    pub const TESSERACT_WEBSITE: &str = "https://github.com/UB-Mannheim/tesseract/wiki";
    #[allow(dead_code)]
    pub const FFMPEG_WEBSITE: &str = "https://ffmpeg.org/download.html";
    #[allow(dead_code)]
    pub const IMAGEMAGICK_WEBSITE: &str = "https://imagemagick.org/script/download.php";
}

// Windows MSI progress reporting
#[cfg(windows)]
mod msi_progress {
    use windows::Win32::System::ApplicationInstallationAndServicing::{
        MsiProcessMessage, MsiCreateRecord, MsiRecordSetStringW, MsiCloseHandle,
        INSTALLMESSAGE, INSTALLMESSAGE_ACTIONSTART, INSTALLMESSAGE_ACTIONDATA, MSIHANDLE
    };
    use windows::core::PCWSTR;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    /// Send an action start message to the MSI installer UI
    pub fn report_action_start(action_name: &str) {
        let _ = send_message(INSTALLMESSAGE_ACTIONSTART, action_name);
    }

    /// Send action data (progress message) to the MSI installer UI
    pub fn report_action_data(message: &str) {
        let _ = send_message(INSTALLMESSAGE_ACTIONDATA, message);
    }

    fn send_message(message_type: INSTALLMESSAGE, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Convert Rust string to wide string for Windows API
        let wide_text: Vec<u16> = OsStr::new(text)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Get the install handle from the MSIHANDLE environment variable
        // This is set by the MSI installer when running custom actions
        if let Ok(handle_str) = std::env::var("MSIHANDLE") {
            if let Ok(handle_value) = handle_str.parse::<u32>() {
                unsafe {
                    let install_handle = MSIHANDLE(handle_value);

                    // Create a record with 1 field for the message
                    let record = MsiCreateRecord(1);
                    if record.0 != 0 {
                        // Set the message text in field 0 (the template field)
                        MsiRecordSetStringW(record, 0, PCWSTR(wide_text.as_ptr()));

                        // Send the message to the installer UI
                        MsiProcessMessage(install_handle, message_type, record);

                        // Clean up the record
                        MsiCloseHandle(record);
                    }
                }
            }
        }

        Ok(())
    }
}

// Stub for non-Windows platforms
#[cfg(not(windows))]
mod msi_progress {
    pub fn report_action_start(_action_name: &str) {}
    pub fn report_action_data(_message: &str) {}
}

/// Centralized progress reporting for dependency installation
struct ProgressReporter<'a> {
    callback: &'a Option<ProgressCallback>,
}

impl<'a> ProgressReporter<'a> {
    fn new(callback: &'a Option<ProgressCallback>) -> Self {
        Self { callback }
    }

    /// Report installation progress with message and percentage
    fn report(&self, message: &str, percentage: u8) {
        // Always report to MSI progress (noop on non-Windows)
        msi_progress::report_action_data(message);

        // Report via callback or println
        if let Some(ref cb) = self.callback {
            cb(message, percentage);
        } else {
            println!("[{}%] {}", percentage, message);
        }
    }

    /// Report action start (primarily for MSI)
    fn report_action(&self, action_name: &str) {
        msi_progress::report_action_start(action_name);
    }
}

/// Represents a dependency and its installation status
#[derive(Debug, Clone, Copy)]
pub struct Dependency {
    pub name: &'static str,
    pub command: &'static str,
    pub required: bool,
    pub description: &'static str,
}

/// List of all dependencies
pub const DEPENDENCIES: &[Dependency] = &[
    Dependency {
        name: "ExifTool",
        command: "exiftool",
        required: true,
        description: "Required for extracting metadata from files",
    },
    Dependency {
        name: "Tesseract OCR",
        command: "tesseract",
        required: false,
        description: "Optional - enables OCR for images without metadata",
    },
    Dependency {
        name: "FFmpeg",
        command: "ffmpeg",
        required: false,
        description: "Optional - enables OCR on video frames",
    },
    Dependency {
        name: "ImageMagick",
        command: "magick",
        required: false,
        description: "Optional - enables HEIC image support on Windows/Linux",
    },
];

/// Checks if a command is available in the system PATH
pub fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Checks the status of all dependencies
pub fn check_dependencies() -> Vec<(Dependency, bool)> {
    DEPENDENCIES
        .iter()
        .map(|dep| (*dep, is_command_available(dep.command)))
        .collect()
}

/// Prints dependency status in a formatted table
pub fn print_dependency_status() {
    println!("\n==================================================");
    println!("  Dependency Status");
    println!("==================================================\n");

    let statuses = check_dependencies();
    let mut all_required_installed = true;

    for (dep, installed) in &statuses {
        let status = if *installed { "✓" } else { "✗" };
        let required_label = if dep.required {
            "[REQUIRED]"
        } else {
            "[OPTIONAL]"
        };

        println!("{} {} {}", status, dep.name, required_label);
        println!("   {}", dep.description);

        if dep.required && !installed {
            all_required_installed = false;
        }

        println!();
    }

    println!("==================================================\n");

    if !all_required_installed {
        println!("⚠ WARNING: Some required dependencies are missing!");
        println!("Run 'nameback --install-deps' to install them.\n");
    }
}

/// Progress callback for dependency installation
pub type ProgressCallback = Box<dyn Fn(&str, u8) + Send + Sync>;

/// Runs the appropriate installer script based on the platform
pub fn run_installer() -> Result<(), String> {
    run_installer_with_progress(None)
}

/// Runs the installer with optional progress callback
/// Callback receives: (status_message, percentage)
pub fn run_installer_with_progress(progress: Option<ProgressCallback>) -> Result<(), String> {
    let is_interactive = progress.is_none();

    // Use Arc to share progress callback across closures (makes it Sync)
    // AtomicBool for thread-safe header_printed flag
    let progress_arc = Arc::new(progress);
    let header_printed = Arc::new(AtomicBool::new(false));

    let progress_arc_clone = Arc::clone(&progress_arc);
    let header_printed_clone = Arc::clone(&header_printed);

    let report_progress = move |msg: &str, pct: u8| {
        // Print header on first call with pct == 0 in interactive mode
        if pct == 0 && is_interactive && !header_printed_clone.swap(true, Ordering::Relaxed) {
            println!("\n==================================================");
            println!("  Installing Dependencies");
            println!("==================================================\n");
        }

        // Always report to MSI progress (noop on non-Windows)
        msi_progress::report_action_data(msg);

        // Report via callback or println
        if let Some(ref cb) = *progress_arc_clone {
            cb(msg, pct);
        } else {
            println!("[{}%] {}", pct, msg);
        }
    };

    // Report action start separately (doesn't need to be in closure)
    msi_progress::report_action_start("Installing nameback dependencies");

    // Print version information at the start
    println!("=== NAMEBACK DEPENDENCY INSTALLER ===");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("======================================\n");

    report_progress("Starting installation...", 0);

    #[cfg(target_os = "windows")]
    {
        // Helper function to temporarily switch to public DNS servers
        // Returns original DNS settings for restoration
        // Ensure Scoop is installed and get the path to scoop.cmd
        let scoop_cmd = windows::ensure_scoop_installed(report_progress)?;

        // Install all Windows dependencies via Scoop (with Chocolatey/bundled fallbacks)
        windows::install_dependencies_via_scoop(&scoop_cmd, report_progress)?;

        // Ensure DNS is restored even if we didn't explicitly restore it earlier
        windows::restore_dns();
    }

    #[cfg(target_os = "macos")]
    {
        // Install all macOS dependencies via Homebrew/MacPorts
        macos::install_dependencies(&report_progress)?;

        // Ensure DNS is restored
        macos::restore_dns();
    }

    #[cfg(target_os = "linux")]
    {
        linux::install_dependencies(&report_progress)?;

        // Ensure DNS is restored
        linux::restore_dns();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err("Unsupported platform. Please install dependencies manually.".to_string());
    }

    if is_interactive {
        println!("\n==================================================");
        println!("  Installation Complete!");
        println!("==================================================\n");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_command_available_for_existing_command() {
        // Test with commands that support --version on all platforms
        #[cfg(unix)]
        {
            // Try common commands that support --version
            let has_bash = is_command_available("bash");
            let has_sh = is_command_available("sh");
            let has_git = is_command_available("git");
            // At least one of these should be available
            assert!(has_bash || has_sh || has_git, "No common commands found");
        }

        #[cfg(windows)]
        {
            // PowerShell should be available on Windows
            assert!(is_command_available("powershell"));
        }
    }

    #[test]
    fn test_is_command_available_for_nonexistent_command() {
        // Test with a command that definitely doesn't exist
        assert!(!is_command_available("this_command_definitely_does_not_exist_12345"));
    }

    #[test]
    fn test_check_dependencies_returns_results() {
        // Test that check_dependencies returns a non-empty vector
        let deps = check_dependencies();
        assert!(!deps.is_empty());

        // Verify structure - should have at least exiftool
        assert!(deps.iter().any(|(dep, _)| dep.name == "ExifTool"));
    }

    #[test]
    fn test_dependencies_have_valid_names() {
        // Test that all dependencies in DEPENDENCIES have proper names
        assert_eq!(DEPENDENCIES.len(), 4);

        let exiftool = DEPENDENCIES.iter().find(|d| d.name == "ExifTool");
        assert!(exiftool.is_some());
        assert_eq!(exiftool.unwrap().command, "exiftool");
        assert!(exiftool.unwrap().required);

        let tesseract = DEPENDENCIES.iter().find(|d| d.name == "Tesseract OCR");
        assert!(tesseract.is_some());
        assert_eq!(tesseract.unwrap().command, "tesseract");
        assert!(!tesseract.unwrap().required);
    }

    #[test]
    fn test_dependencies_have_descriptions() {
        // Test that all dependencies have non-empty descriptions
        for dep in DEPENDENCIES {
            assert!(!dep.description.is_empty());
            assert!(!dep.name.is_empty());
            assert!(!dep.command.is_empty());
        }
    }

    #[test]
    fn test_exiftool_is_required() {
        // ExifTool should always be marked as required
        let exiftool = DEPENDENCIES.iter().find(|d| d.name == "ExifTool");
        assert!(exiftool.is_some());
        assert!(exiftool.unwrap().required);
    }

    #[test]
    fn test_optional_dependencies() {
        // Tesseract, FFmpeg, and ImageMagick should be optional
        let optional_count = DEPENDENCIES.iter().filter(|d| !d.required).count();
        assert_eq!(optional_count, 3);
    }
}
