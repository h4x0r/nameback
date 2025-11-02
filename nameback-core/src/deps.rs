use std::process::Command;

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
pub type ProgressCallback = Box<dyn Fn(&str, u8) + Send>;

/// Runs the appropriate installer script based on the platform
pub fn run_installer() -> Result<(), String> {
    run_installer_with_progress(None)
}

/// Runs the installer with optional progress callback
/// Callback receives: (status_message, percentage)
pub fn run_installer_with_progress(progress: Option<ProgressCallback>) -> Result<(), String> {
    let is_interactive = progress.is_none();
    let reporter = ProgressReporter::new(&progress);
    let report_progress = |msg: &str, pct: u8| {
        if pct == 0 && is_interactive {
            println!("\n==================================================");
            println!("  Installing Dependencies");
            println!("==================================================\n");
        }
        reporter.report(msg, pct);
    };

    reporter.report_action("Installing nameback dependencies");

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
        // Helper function to temporarily switch to public DNS on macOS
        let try_with_public_dns_macos = || -> Result<(), String> {
            println!("\n=== DNS FALLBACK (macOS): Attempting to use public DNS servers ===");

            // Get list of active network services
            let services_output = Command::new("networksetup")
                .arg("-listallnetworkservices")
                .output()
                .map_err(|e| format!("Failed to list network services: {}", e))?;

            if !services_output.status.success() {
                return Err("Failed to list network services".to_string());
            }

            let services = String::from_utf8_lossy(&services_output.stdout);
            let active_services: Vec<&str> = services
                .lines()
                .skip(1) // Skip the asterisk line
                .filter(|line| !line.starts_with('*'))
                .collect();

            // Save original DNS settings for each service
            let mut original_dns = Vec::new();
            for service in &active_services {
                if let Ok(output) = Command::new("networksetup")
                    .args(["-getdnsservers", service])
                    .output()
                {
                    let dns = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("Saved DNS for {}: {}", service, dns);
                    original_dns.push((service.to_string(), dns));
                }
            }

            // Store original DNS in environment variable for restoration
            let dns_json = serde_json::to_string(&original_dns)
                .map_err(|e| format!("Failed to serialize DNS settings: {}", e))?;
            std::env::set_var("NAMEBACK_ORIGINAL_DNS_MACOS", &dns_json);

            // Set public DNS for all services
            for service in &active_services {
                println!("Setting public DNS for {}...", service);
                let _ = Command::new("networksetup")
                    .args(["-setdnsservers", service, "8.8.8.8", "8.8.4.4", "1.1.1.1"])
                    .status();
            }

            println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
            Ok(())
        };

        // Helper function to restore original DNS settings on macOS
        let restore_dns_macos = || {
            if let Ok(dns_json) = std::env::var("NAMEBACK_ORIGINAL_DNS_MACOS") {
                println!("\n=== DNS FALLBACK (macOS): Restoring original DNS settings ===");

                if let Ok(original_dns) = serde_json::from_str::<Vec<(String, String)>>(&dns_json) {
                    for (service, dns) in original_dns {
                        println!("Restoring DNS for {}: {}", service, dns);
                        if dns.contains("There aren't any DNS Servers") || dns.is_empty() {
                            // Reset to DHCP
                            let _ = Command::new("networksetup")
                                .args(["-setdnsservers", &service, "empty"])
                                .status();
                        } else {
                            // Restore specific DNS servers
                            let servers: Vec<&str> = dns.split('\n').collect();
                            let mut cmd = Command::new("networksetup");
                            cmd.arg("-setdnsservers").arg(&service);
                            for server in servers {
                                if !server.trim().is_empty() {
                                    cmd.arg(server.trim());
                                }
                            }
                            let _ = cmd.status();
                        }
                    }
                }

                println!("DNS settings restored");
                std::env::remove_var("NAMEBACK_ORIGINAL_DNS_MACOS");
            }
        };

        report_progress("Checking Homebrew installation...", 10);

        // Check if Homebrew is installed
        let brew_check = Command::new("brew")
            .arg("--version")
            .output();

        let brew_installed = brew_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !brew_installed {
            println!("Homebrew not found. Checking for MacPorts as fallback...");

            // Try MacPorts as fallback
            let port_check = Command::new("port")
                .arg("version")
                .output();

            let port_installed = port_check
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !port_installed {
                return Err(
                    "No package manager found. Please install Homebrew (https://brew.sh) or MacPorts (https://www.macports.org)".to_string()
                );
            }

            println!("MacPorts found, using as fallback package manager");
            // We'll use MacPorts below
        }

        // Helper to install with Homebrew with DNS fallback
        let install_with_brew = |package: &str| -> bool {
            println!("Installing {} with Homebrew...", package);
            let result = Command::new("brew")
                .args(["install", package])
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    println!("{} installed successfully", package);
                    true
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Homebrew installation failed for {}: {}", package, stderr);

                    // Check for DNS/network errors
                    if stderr.contains("Could not resolve") ||
                       stderr.contains("Failed to connect") ||
                       stderr.contains("curl") && stderr.contains("error") {
                        println!("Detected network error, trying DNS fallback...");

                        if try_with_public_dns_macos().is_ok() {
                            println!("Retrying {} installation with public DNS...", package);
                            let retry = Command::new("brew")
                                .args(["install", package])
                                .output();

                            restore_dns_macos();

                            if let Ok(retry_output) = retry {
                                if retry_output.status.success() {
                                    println!("{} installed successfully with DNS fallback", package);
                                    return true;
                                }
                            }
                        }
                    }
                    false
                }
                Err(e) => {
                    println!("Failed to execute brew command: {}", e);
                    false
                }
            }
        };

        // Helper to install with MacPorts as fallback
        let install_with_port = |package: &str| -> bool {
            println!("Trying MacPorts as fallback for {}...", package);
            let result = Command::new("sudo")
                .args(["port", "install", package])
                .status();

            match result {
                Ok(status) if status.success() => {
                    println!("{} installed successfully via MacPorts", package);
                    true
                }
                _ => {
                    println!("MacPorts installation also failed for {}", package);
                    false
                }
            }
        };

        report_progress("Installing exiftool (required)...", 30);

        let exiftool_installed = if brew_installed {
            install_with_brew("exiftool") || install_with_port("exiftool")
        } else {
            install_with_port("exiftool")
        };

        if !exiftool_installed {
            restore_dns_macos();
            return Err("Failed to install exiftool. Please install manually: brew install exiftool".to_string());
        }

        report_progress("Installing tesseract (optional OCR support)...", 50);
        let tesseract_installed = if brew_installed {
            install_with_brew("tesseract") || install_with_brew("tesseract-lang")
        } else {
            install_with_port("tesseract")
        };

        if !tesseract_installed {
            println!("WARNING: Tesseract (OCR) installation failed (optional)");
            println!("  OCR support will be disabled");
            println!("  Install manually: brew install tesseract tesseract-lang");
        }

        report_progress("Installing ffmpeg (optional video support)...", 70);
        let ffmpeg_installed = if brew_installed {
            install_with_brew("ffmpeg")
        } else {
            install_with_port("ffmpeg")
        };

        if !ffmpeg_installed {
            println!("WARNING: FFmpeg installation failed (optional)");
            println!("  Video frame extraction will be disabled");
            println!("  Install manually: brew install ffmpeg");
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        let imagemagick_installed = if brew_installed {
            install_with_brew("imagemagick")
        } else {
            install_with_port("ImageMagick")
        };

        if !imagemagick_installed {
            println!("WARNING: ImageMagick installation failed (optional)");
            println!("  HEIC/HEIF image support will be disabled");
            println!("  Install manually: brew install imagemagick");
        }

        // Ensure DNS is restored
        restore_dns_macos();

        report_progress("macOS dependencies installed", 100);
    }

    #[cfg(target_os = "linux")]
    {
        // Helper function to temporarily switch to public DNS on Linux
        let try_with_public_dns_linux = || -> Result<(), String> {
            println!("\n=== DNS FALLBACK (Linux): Attempting to use public DNS servers ===");

            // Save original resolv.conf
            let resolv_conf_path = "/etc/resolv.conf";
            let original_resolv = std::fs::read_to_string(resolv_conf_path)
                .unwrap_or_else(|_| String::new());

            std::env::set_var("NAMEBACK_ORIGINAL_RESOLV_CONF", &original_resolv);

            // Create temporary resolv.conf with public DNS
            let new_resolv = "# Temporary public DNS for Nameback installation\nnameserver 8.8.8.8\nnameserver 8.8.4.4\nnameserver 1.1.1.1\n";

            // Try to write new resolv.conf (requires root/sudo)
            if let Err(e) = Command::new("sudo")
                .arg("sh")
                .arg("-c")
                .arg(format!("echo '{}' > {}", new_resolv, resolv_conf_path))
                .status()
            {
                return Err(format!("Failed to update DNS: {}", e));
            }

            println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
            Ok(())
        };

        // Helper function to restore original DNS settings on Linux
        let restore_dns_linux = || {
            if let Ok(original_resolv) = std::env::var("NAMEBACK_ORIGINAL_RESOLV_CONF") {
                println!("\n=== DNS FALLBACK (Linux): Restoring original DNS settings ===");

                if !original_resolv.is_empty() {
                    // Restore original resolv.conf
                    let _ = Command::new("sudo")
                        .arg("sh")
                        .arg("-c")
                        .arg(format!("echo '{}' > /etc/resolv.conf", original_resolv))
                        .status();
                }

                println!("DNS settings restored");
                std::env::remove_var("NAMEBACK_ORIGINAL_RESOLV_CONF");
            }
        };

        report_progress("Detecting package managers...", 10);

        // Detect ALL available package managers for fallback
        let mut available_managers = Vec::new();

        if Command::new("apt-get").arg("--version").output().is_ok() {
            available_managers.push(("apt-get", vec!["install", "-y"], "libimage-exiftool-perl"));
        }
        if Command::new("dnf").arg("--version").output().is_ok() {
            available_managers.push(("dnf", vec!["install", "-y"], "perl-Image-ExifTool"));
        }
        if Command::new("yum").arg("--version").output().is_ok() {
            available_managers.push(("yum", vec!["install", "-y"], "perl-Image-ExifTool"));
        }
        if Command::new("pacman").arg("--version").output().is_ok() {
            available_managers.push(("pacman", vec!["-S", "--noconfirm"], "perl-image-exiftool"));
        }
        if Command::new("snap").arg("version").output().is_ok() {
            available_managers.push(("snap", vec!["install"], "exiftool"));
        }

        if available_managers.is_empty() {
            return Err("No supported package manager found (apt-get, dnf, yum, pacman, or snap required)".to_string());
        }

        println!("Found {} package manager(s): {}",
                 available_managers.len(),
                 available_managers.iter().map(|(name, _, _)| *name).collect::<Vec<_>>().join(", "));

        // Check if running with sudo/root
        let needs_sudo = std::env::var("USER").unwrap_or_default() != "root";

        // Helper function to try installing with all available package managers
        let try_install_package = |display_name: &str, packages: &[(&str, &str)]| -> bool {
            println!("Installing {}...", display_name);

            // Try each available package manager
            for (pkg_manager, install_cmd, _) in &available_managers {
                // Find package name for this manager
                let package = packages.iter()
                    .find(|(manager, _)| manager == pkg_manager)
                    .map(|(_, pkg)| *pkg);

                if let Some(package) = package {
                    println!("Trying {} with {}...", display_name, pkg_manager);

                    let result = if *pkg_manager == "snap" {
                        // snap doesn't need sudo prefix in command
                        let mut cmd = Command::new("sudo");
                        cmd.arg("snap");
                        for arg in install_cmd {
                            cmd.arg(arg);
                        }
                        cmd.arg(package).output()
                    } else if needs_sudo {
                        let mut cmd = Command::new("sudo");
                        cmd.arg(pkg_manager);
                        for arg in install_cmd {
                            cmd.arg(arg);
                        }
                        cmd.arg(package).output()
                    } else {
                        let mut cmd = Command::new(pkg_manager);
                        for arg in install_cmd {
                            cmd.arg(arg);
                        }
                        cmd.arg(package).output()
                    };

                    match result {
                        Ok(output) if output.status.success() => {
                            println!("{} installed successfully with {}", display_name, pkg_manager);
                            return true;
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            println!("{} failed with {}: {}", display_name, pkg_manager, stderr);

                            // Check for DNS/network errors
                            if stderr.contains("Could not resolve") ||
                               stderr.contains("Temporary failure in name resolution") ||
                               stderr.contains("Name or service not known") {
                                println!("Detected DNS error, trying DNS fallback...");

                                if try_with_public_dns_linux().is_ok() {
                                    println!("Retrying {} with public DNS...", display_name);

                                    let retry = if *pkg_manager == "snap" {
                                        let mut cmd = Command::new("sudo");
                                        cmd.arg("snap");
                                        for arg in install_cmd {
                                            cmd.arg(arg);
                                        }
                                        cmd.arg(package).output()
                                    } else if needs_sudo {
                                        let mut cmd = Command::new("sudo");
                                        cmd.arg(pkg_manager);
                                        for arg in install_cmd {
                                            cmd.arg(arg);
                                        }
                                        cmd.arg(package).output()
                                    } else {
                                        let mut cmd = Command::new(pkg_manager);
                                        for arg in install_cmd {
                                            cmd.arg(arg);
                                        }
                                        cmd.arg(package).output()
                                    };

                                    restore_dns_linux();

                                    if let Ok(retry_output) = retry {
                                        if retry_output.status.success() {
                                            println!("{} installed successfully with DNS fallback", display_name);
                                            return true;
                                        }
                                    }
                                }
                            }

                            // Try next package manager
                            continue;
                        }
                        Err(e) => {
                            println!("Failed to execute {} command: {}", pkg_manager, e);
                            continue;
                        }
                    }
                }
            }

            false
        };

        report_progress("Installing exiftool (required)...", 30);

        let exiftool_packages = vec![
            ("apt-get", "libimage-exiftool-perl"),
            ("dnf", "perl-Image-ExifTool"),
            ("yum", "perl-Image-ExifTool"),
            ("pacman", "perl-image-exiftool"),
            ("snap", "exiftool"),
        ];

        if !try_install_package("exiftool", &exiftool_packages) {
            restore_dns_linux();
            return Err("Failed to install exiftool with all available package managers".to_string());
        }

        report_progress("Installing tesseract (optional OCR support)...", 50);

        let tesseract_packages = vec![
            ("apt-get", "tesseract-ocr"),
            ("dnf", "tesseract"),
            ("yum", "tesseract"),
            ("pacman", "tesseract"),
            ("snap", "tesseract"),
        ];

        if !try_install_package("tesseract", &tesseract_packages) {
            println!("WARNING: Tesseract (OCR) installation failed (optional)");
            println!("  OCR support will be disabled");
            println!("  Install manually with your package manager");
        }

        report_progress("Installing ffmpeg (optional video support)...", 70);

        let ffmpeg_packages = vec![
            ("apt-get", "ffmpeg"),
            ("dnf", "ffmpeg"),
            ("yum", "ffmpeg"),
            ("pacman", "ffmpeg"),
            ("snap", "ffmpeg"),
        ];

        if !try_install_package("ffmpeg", &ffmpeg_packages) {
            println!("WARNING: FFmpeg installation failed (optional)");
            println!("  Video frame extraction will be disabled");
            println!("  Install manually with your package manager");
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);

        let imagemagick_packages = vec![
            ("apt-get", "imagemagick"),
            ("dnf", "ImageMagick"),
            ("yum", "ImageMagick"),
            ("pacman", "imagemagick"),
            ("snap", "imagemagick"),
        ];

        if !try_install_package("imagemagick", &imagemagick_packages) {
            println!("WARNING: ImageMagick installation failed (optional)");
            println!("  HEIC/HEIF image support will be disabled");
            println!("  Install manually with your package manager");
        }

        // Ensure DNS is restored
        restore_dns_linux();

        report_progress("Linux dependencies installed", 100);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err("Unsupported platform. Please install dependencies manually.".to_string());
    }

    if progress.is_none() {
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
