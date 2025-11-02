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

    // Helper function to download and install from bundled GitHub Release assets
    #[allow(unused_variables)] // Used only in Windows-specific code paths
    let install_from_bundled = |dep_name: &str, platform: &str| -> Result<(), String> {
        let version = env!("CARGO_PKG_VERSION");
        let asset_name = format!("deps-{}-{}.zip", dep_name, platform);
        let download_url = format!(
            "{}/v{}/{}",
            constants::GITHUB_RELEASES_BASE, version, asset_name
        );

        println!("\n=== BUNDLED INSTALLER FALLBACK ===");
        println!("Downloading {} from GitHub Release...", dep_name);
        println!("URL: {}", download_url);

        // Use reqwest to download (already in dependencies)
        let response = reqwest::blocking::get(&download_url)
            .map_err(|e| format!("Failed to download bundled installer: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub Release asset not found: {} (HTTP {})", asset_name, response.status()));
        }

        // Save to temp directory
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join(&asset_name);
        let content = response.bytes()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        std::fs::write(&installer_path, &content)
            .map_err(|e| format!("Failed to write installer: {}", e))?;

        println!("Downloaded to: {}", installer_path.display());
        println!("Installing {}...", dep_name);

        // Platform-specific extraction and installation
        #[cfg(target_os = "windows")]
        {
            // Extract zip and run installer
            let extract_dir = temp_dir.join(format!("{}-extract", dep_name));
            std::fs::create_dir_all(&extract_dir)
                .map_err(|e| format!("Failed to create extract dir: {}", e))?;

            // Use PowerShell to extract
            let extract_cmd = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                installer_path.display(),
                extract_dir.display()
            );

            let extract_result = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&extract_cmd)
                .output();

            if let Ok(output) = extract_result {
                if !output.status.success() {
                    return Err(format!("Failed to extract: {}", String::from_utf8_lossy(&output.stderr)));
                }
            } else {
                return Err("Failed to run extraction command".to_string());
            }

            // Run the installer based on dependency type
            match dep_name {
                "tesseract" => {
                    // Find and run the setup.exe
                    let setup_exe = extract_dir.join("tesseract-windows-setup.exe");
                    if setup_exe.exists() {
                        Command::new(&setup_exe)
                            .arg("/S")  // Silent install
                            .status()
                            .map_err(|e| format!("Failed to run installer: {}", e))?;
                    }
                }
                "exiftool" | "ffmpeg" | "imagemagick" => {
                    // For portable versions, copy to a standard location
                    let install_dir = std::env::var("LOCALAPPDATA")
                        .unwrap_or_else(|_| "C:\\Program Files".to_string());
                    let target_dir = std::path::PathBuf::from(&install_dir).join("Nameback").join(dep_name);

                    std::fs::create_dir_all(&target_dir)
                        .map_err(|e| format!("Failed to create install dir: {}", e))?;

                    // Copy files
                    let copy_cmd = format!(
                        "Copy-Item -Path '{}\\*' -Destination '{}' -Recurse -Force",
                        extract_dir.display(),
                        target_dir.display()
                    );

                    Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg(&copy_cmd)
                        .output()
                        .map_err(|e| format!("Failed to copy files: {}", e))?;

                    println!("Installed to: {}", target_dir.display());
                    println!("Note: You may need to add this to your PATH manually.");
                }
                _ => return Err(format!("Unknown dependency: {}", dep_name)),
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS-specific installation
            let extract_dir = temp_dir.join(format!("{}-extract", dep_name));
            std::fs::create_dir_all(&extract_dir)
                .map_err(|e| format!("Failed to create extract dir: {}", e))?;

            // Extract tar.gz or dmg
            if installer_path.extension().and_then(|s| s.to_str()) == Some("dmg") {
                // Mount DMG and copy
                Command::new("hdiutil")
                    .args(["attach", installer_path.to_str().unwrap()])
                    .output()
                    .map_err(|e| format!("Failed to mount DMG: {}", e))?;
            } else {
                // Extract archive
                Command::new("unzip")
                    .args(["-q", installer_path.to_str().unwrap(), "-d", extract_dir.to_str().unwrap()])
                    .output()
                    .map_err(|e| format!("Failed to extract: {}", e))?;
            }

            println!("Extracted. Manual installation may be required.");
        }

        #[cfg(target_os = "linux")]
        {
            // Linux-specific installation
            let extract_dir = temp_dir.join(format!("{}-extract", dep_name));
            std::fs::create_dir_all(&extract_dir)
                .map_err(|e| format!("Failed to create extract dir: {}", e))?;

            // Extract tar.gz
            Command::new("tar")
                .args(&["-xzf", installer_path.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
                .output()
                .map_err(|e| format!("Failed to extract: {}", e))?;

            println!("Extracted to: {}", extract_dir.display());
            println!("Note: Manual installation of .deb packages may be required.");
        }

        Ok(())
    };

    #[cfg(target_os = "windows")]
    {
        // Helper function to temporarily switch to public DNS servers
        // Returns original DNS settings for restoration
        let try_with_public_dns = || -> Result<(), String> {
            println!("\n=== DNS FALLBACK: Attempting to use public DNS servers ===");

            // PowerShell script to save current DNS, switch to public DNS, and return original settings
            let setup_dns_script = r#"
                # Get all active network adapters
                $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
                $originalDNS = @()

                foreach ($adapter in $adapters) {
                    # Save current DNS settings
                    $currentDNS = Get-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -AddressFamily IPv4
                    $originalDNS += [PSCustomObject]@{
                        InterfaceIndex = $adapter.InterfaceIndex
                        InterfaceAlias = $adapter.InterfaceAlias
                        ServerAddresses = $currentDNS.ServerAddresses
                    }

                    # Set public DNS (Google 8.8.8.8, 8.8.4.4, Cloudflare 1.1.1.1)
                    try {
                        Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses ("8.8.8.8","8.8.4.4","1.1.1.1")
                        Write-Host "Set public DNS for $($adapter.InterfaceAlias)"
                    } catch {
                        Write-Warning "Failed to set DNS for $($adapter.InterfaceAlias): $_"
                    }
                }

                # Return original DNS settings as JSON for restoration
                $originalDNS | ConvertTo-Json -Compress
            "#;

            let output = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(setup_dns_script)
                .output()
                .map_err(|e| format!("Failed to execute DNS setup script: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("DNS setup failed: {}", stderr));
            }

            let original_dns_json = String::from_utf8_lossy(&output.stdout);
            println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
            println!("Original DNS settings saved for restoration");

            // Store original DNS for restoration (we'll use this later)
            std::env::set_var("NAMEBACK_ORIGINAL_DNS", original_dns_json.trim());

            Ok(())
        };

        // Helper function to restore original DNS settings
        let restore_dns = || {
            if let Ok(original_dns_json) = std::env::var("NAMEBACK_ORIGINAL_DNS") {
                println!("\n=== DNS FALLBACK: Restoring original DNS settings ===");

                let restore_script = format!(r#"
                    $originalDNS = '{}' | ConvertFrom-Json

                    foreach ($dns in $originalDNS) {{
                        try {{
                            if ($dns.ServerAddresses -and $dns.ServerAddresses.Count -gt 0) {{
                                Set-DnsClientServerAddress -InterfaceIndex $dns.InterfaceIndex -ServerAddresses $dns.ServerAddresses
                                Write-Host "Restored DNS for $($dns.InterfaceAlias)"
                            }} else {{
                                # No DNS was set (DHCP), reset to automatic
                                Set-DnsClientServerAddress -InterfaceIndex $dns.InterfaceIndex -ResetServerAddresses
                                Write-Host "Reset DNS to DHCP for $($dns.InterfaceAlias)"
                            }}
                        }} catch {{
                            Write-Warning "Failed to restore DNS for $($dns.InterfaceAlias): $_"
                        }}
                    }}
                "#, original_dns_json);

                let _ = Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-Command")
                    .arg(&restore_script)
                    .output();

                println!("DNS settings restored");
                std::env::remove_var("NAMEBACK_ORIGINAL_DNS");
            }
        };

        report_progress("Checking Scoop installation...", 10);

        // Get USERPROFILE path for Scoop installation location
        let user_profile = std::env::var("USERPROFILE")
            .map_err(|_| "USERPROFILE environment variable not set".to_string())?;

        println!("=== DEBUG: Environment Information ===");
        println!("USERPROFILE: {}", user_profile);
        println!("COMSPEC: {}", std::env::var("COMSPEC").unwrap_or_else(|_| "<not set>".to_string()));
        println!("PATH: {}", std::env::var("PATH").unwrap_or_else(|_| "<not set>".to_string()));

        // Check if Scoop is installed
        println!("=== DEBUG: Checking Scoop Installation ===");
        println!("Command: powershell -NoProfile -Command \"Get-Command scoop -ErrorAction SilentlyContinue\"");

        let scoop_check = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("Get-Command scoop -ErrorAction SilentlyContinue")
            .output();

        match &scoop_check {
            Ok(output) => {
                println!("Scoop check exit code: {:?}", output.status.code());
                println!("Scoop check stdout: {}", String::from_utf8_lossy(&output.stdout));
                println!("Scoop check stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                eprintln!("Failed to run scoop check command: {}", e);
            }
        }

        let scoop_installed = scoop_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        println!("Scoop installed: {}", scoop_installed);

        if !scoop_installed {
            report_progress("Installing Scoop package manager (using admin rights)...", 20);

            // Use a temp file approach to avoid the Security module issue
            // Download installer to temp, execute with -RunAsAdmin (we have UAC already)
            let temp_dir = std::env::var("TEMP").unwrap_or_else(|_| format!("{}\\AppData\\Local\\Temp", user_profile));
            let installer_path = format!("{}\\install-scoop.ps1", temp_dir);

            println!("=== DEBUG: Installing Scoop ===");
            println!("Temp installer path: {}", installer_path);

            // Download the Scoop installer to a temp file
            let download_cmd = format!(
                "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
                constants::SCOOP_INSTALL,
                installer_path
            );

            println!("Download command: powershell -NoProfile -Command \"{}\"", download_cmd);

            let download_result = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(&download_cmd)
                .output()
                .map_err(|e| {
                    eprintln!("Failed to execute PowerShell command to download Scoop installer: {}", e);
                    format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  POWERSHELL EXECUTION ERROR                                      ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         Unable to execute PowerShell to download Scoop installer.\n\n\
                         Error: {}\n\n\
                         This may indicate:\n\
                         • PowerShell is not available or not in PATH\n\
                         • Execution policies are too restrictive\n\
                         • System resource limitations\n\n\
                         Please try installing dependencies manually:\n\
                         • Visit https://scoop.sh for Scoop installation\n\
                         • Or download tools directly from their websites\n", e
                    )
                })?;

            println!("Download exit code: {:?}", download_result.status.code());
            println!("Download stdout: {}", String::from_utf8_lossy(&download_result.stdout));
            println!("Download stderr: {}", String::from_utf8_lossy(&download_result.stderr));

            if !download_result.status.success() {
                let stderr = String::from_utf8_lossy(&download_result.stderr);
                eprintln!("Failed to download Scoop installer!");
                eprintln!("  stderr: {}", stderr);

                // Check if it's a DNS/network error
                let is_dns_error = stderr.contains("could not be resolved") ||
                                   stderr.contains("unable to resolve") ||
                                   stderr.contains("DNS");
                let is_network_error = stderr.contains("Unable to connect") ||
                                       stderr.contains("connection") ||
                                       stderr.contains("network");

                if is_dns_error || is_network_error {
                    println!("Detected DNS/network error, attempting DNS fallback to public DNS servers...");
                    msi_progress::report_action_data("DNS error detected, trying public DNS servers...");

                    // Try switching to public DNS and retrying
                    if let Ok(()) = try_with_public_dns() {
                        println!("Retrying Scoop installer download with public DNS...");

                        let download_retry = Command::new("powershell")
                            .arg("-NoProfile")
                            .arg("-Command")
                            .arg(&download_cmd)
                            .output();

                        // Restore DNS regardless of outcome
                        restore_dns();

                        match download_retry {
                            Ok(output) if output.status.success() => {
                                println!("Scoop installer downloaded successfully with public DNS!");
                                // Continue with installation - the download succeeded
                            }
                            _ => {
                                let error_msg = format!(
                                    "\n╔══════════════════════════════════════════════════════════════════╗\n\
                                     ║  NETWORK CONNECTION ERROR                                        ║\n\
                                     ╚══════════════════════════════════════════════════════════════════╝\n\n\
                                     Unable to download Scoop installer even with public DNS fallback.\n\n\
                                     Possible causes:\n\
                                     • Complete network outage\n\
                                     • Firewall or proxy blocking all connections\n\
                                     • VPN interference\n\n\
                                     Manual installation option:\n\
                                     You can install dependencies manually:\n\
                                     • Visit https://scoop.sh for Scoop installation\n\
                                     • After Scoop is installed, run:\n\
                                       scoop install exiftool tesseract ffmpeg imagemagick\n\n\
                                     Or download dependencies directly:\n\
                                     • ExifTool: https://exiftool.org/\n\
                                     • Tesseract: https://github.com/UB-Mannheim/tesseract/wiki\n\
                                     • FFmpeg: https://ffmpeg.org/download.html\n\
                                     • ImageMagick: https://imagemagick.org/script/download.php\n"
                                );
                                return Err(error_msg);
                            }
                        }
                    } else {
                        println!("WARNING: Could not switch to public DNS, continuing with original error...");
                        let error_msg = format!(
                            "\n╔══════════════════════════════════════════════════════════════════╗\n\
                             ║  NETWORK CONNECTION ERROR                                        ║\n\
                             ╚══════════════════════════════════════════════════════════════════╝\n\n\
                             Unable to download Scoop installer due to network issues.\n\n\
                             Possible causes:\n\
                             • DNS resolution failure (cannot resolve 'get.scoop.sh')\n\
                             • Network connectivity problems\n\
                             • Firewall or proxy blocking the connection\n\
                             • VPN interference\n\n\
                             Troubleshooting steps:\n\
                             1. Check your internet connection\n\
                             2. Try accessing https://get.scoop.sh in a web browser\n\
                             3. Check DNS settings (try 8.8.8.8 or 1.1.1.1)\n\
                             4. Disable VPN temporarily and retry\n\
                             5. Check firewall/antivirus settings\n\n\
                             Manual installation option:\n\
                             You can install dependencies manually:\n\
                             • Visit https://scoop.sh for Scoop installation\n\
                             • After Scoop is installed, run:\n\
                               scoop install exiftool tesseract ffmpeg imagemagick\n\n\
                             Or download dependencies directly:\n\
                             • ExifTool: https://exiftool.org/\n\
                             • Tesseract: https://github.com/UB-Mannheim/tesseract/wiki\n\
                             • FFmpeg: https://ffmpeg.org/download.html\n\
                             • ImageMagick: https://imagemagick.org/script/download.php\n"
                        );
                        return Err(error_msg);
                    }
                }

                return Err(format!("Failed to download Scoop installer: {}", stderr));
            }

            // Execute the installer with -RunAsAdmin flag (we already have UAC permission)
            let install_cmd = format!("& '{}' -RunAsAdmin; Remove-Item '{}'", installer_path, installer_path);

            println!("Install command: powershell -NoProfile -ExecutionPolicy Bypass -Command \"{}\"", install_cmd);

            let scoop_install = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg(&install_cmd)
                .output()
                .map_err(|e| {
                    eprintln!("Failed to execute Scoop installer: {}", e);
                    format!("Failed to execute Scoop installer: {}", e)
                })?;

            println!("Scoop install exit code: {:?}", scoop_install.status.code());
            println!("Scoop install stdout: {}", String::from_utf8_lossy(&scoop_install.stdout));
            println!("Scoop install stderr: {}", String::from_utf8_lossy(&scoop_install.stderr));

            if !scoop_install.status.success() {
                let stderr = String::from_utf8_lossy(&scoop_install.stderr);
                let stdout = String::from_utf8_lossy(&scoop_install.stdout);
                eprintln!("Scoop installation failed!");
                eprintln!("  stdout: {}", stdout);
                eprintln!("  stderr: {}", stderr);

                // Check for common failure patterns
                let is_network_related = stderr.contains("could not be resolved") ||
                                         stderr.contains("Unable to connect") ||
                                         stderr.contains("network") ||
                                         stderr.contains("connection");

                if is_network_related {
                    return Err(format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  SCOOP INSTALLATION FAILED - NETWORK ERROR                       ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         The Scoop installer encountered a network error.\n\n\
                         Error details:\n{}\n\n\
                         Please check your network connection and try again.\n\n\
                         Manual installation:\n\
                         1. Fix your network/DNS issues first\n\
                         2. Visit https://scoop.sh and follow installation instructions\n\
                         3. After Scoop is installed, run:\n\
                            scoop install exiftool tesseract ffmpeg imagemagick\n", stderr
                    ));
                }

                return Err(format!(
                    "\n╔══════════════════════════════════════════════════════════════════╗\n\
                     ║  SCOOP INSTALLATION FAILED                                       ║\n\
                     ╚══════════════════════════════════════════════════════════════════╝\n\n\
                     The Scoop package manager failed to install.\n\n\
                     Error details:\n{}\n\n\
                     Manual installation:\n\
                     1. Visit https://scoop.sh for installation instructions\n\
                     2. Open PowerShell and run:\n\
                        Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser\n\
                        Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression\n\
                     3. After Scoop is installed, run:\n\
                        scoop install exiftool tesseract ffmpeg imagemagick\n", stderr
                ));
            }

            println!("Scoop installed successfully to {}", user_profile);
        }

        // After Scoop installation, use the scoop.cmd shim from shims directory
        // The scoop.cmd file is created immediately during Scoop installation
        // Note: .cmd files must be executed via cmd.exe on Windows

        // Use full path to cmd.exe - the installer may not have cmd in PATH
        let cmd_exe = std::env::var("COMSPEC").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".to_string());
        let scoop_cmd = format!("{}\\scoop\\shims\\scoop.cmd", user_profile);

        // Install 7zip first - required for extracting other packages
        report_progress("Installing 7zip (required for extracting packages)...", 30);
        msi_progress::report_action_data("Downloading and installing 7zip...");
        println!("=== DEBUG: Installing 7zip ===");
        println!("Full command: {} /c \"{}\" install 7zip", cmd_exe, scoop_cmd);

        let seven_zip_result = Command::new(&cmd_exe)
            .arg("/c")
            .arg(&scoop_cmd)
            .arg("install")
            .arg("7zip")
            .output()
            .map_err(|e| {
                msi_progress::report_action_data("ERROR: Failed to execute 7zip install command");
                eprintln!("Failed to execute 7zip install command: {}", e);
                format!("Failed to run scoop install 7zip: {}", e)
            })?;

        println!("7zip install exit code: {:?}", seven_zip_result.status.code());
        println!("7zip install stdout: {}", String::from_utf8_lossy(&seven_zip_result.stdout));
        println!("7zip install stderr: {}", String::from_utf8_lossy(&seven_zip_result.stderr));

        if !seven_zip_result.status.success() {
            let stderr = String::from_utf8_lossy(&seven_zip_result.stderr);
            let stdout = String::from_utf8_lossy(&seven_zip_result.stdout);
            msi_progress::report_action_data("ERROR: 7zip installation failed");
            eprintln!("7zip installation failed!");
            eprintln!("  stdout: {}", stdout);
            eprintln!("  stderr: {}", stderr);
            return Err(format!(
                "\n╔══════════════════════════════════════════════════════════════════╗\n\
                 ║  7ZIP INSTALLATION FAILED                                        ║\n\
                 ╚══════════════════════════════════════════════════════════════════╝\n\n\
                 7zip is required to extract other packages.\n\n\
                 Error details:\n{}\n\n\
                 Please try installing manually:\n\
                 • Run: scoop install 7zip\n", stderr
            ));
        }

        msi_progress::report_action_data("7zip installed successfully");
        println!("7zip installed successfully");

        report_progress("Installing exiftool (required)...", 45);
        msi_progress::report_action_data("Downloading and installing exiftool...");
        println!("=== DEBUG: Installing exiftool ===");
        println!("cmd.exe location: {}", cmd_exe);
        println!("scoop.cmd location: {}", scoop_cmd);
        println!("Full command: {} /c \"{}\" install exiftool", cmd_exe, scoop_cmd);

        let exiftool_result = Command::new(&cmd_exe)
            .arg("/c")
            .arg(&scoop_cmd)
            .arg("install")
            .arg("exiftool")
            .output()
            .map_err(|e| {
                msi_progress::report_action_data("ERROR: Failed to execute exiftool install command");
                eprintln!("Failed to execute exiftool install command: {}", e);
                format!("Failed to run scoop install exiftool: {}", e)
            })?;

        println!("exiftool install exit code: {:?}", exiftool_result.status.code());
        println!("exiftool install stdout: {}", String::from_utf8_lossy(&exiftool_result.stdout));
        println!("exiftool install stderr: {}", String::from_utf8_lossy(&exiftool_result.stderr));

        let stdout = String::from_utf8_lossy(&exiftool_result.stdout);
        let stderr = String::from_utf8_lossy(&exiftool_result.stderr);

        // Scoop reports errors in stdout, not stderr, and still exits with code 0
        // Check for common error patterns
        let has_error = !exiftool_result.status.success() ||
                        stdout.contains("is not valid") ||
                        stdout.contains("ERROR") ||
                        stdout.contains("Failed to") ||
                        stdout.contains("Authentication failed") ||
                        stdout.contains("Unable to connect") ||
                        stdout.contains("could not be resolved");

        if has_error {
            msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
            eprintln!("exiftool installation via Scoop failed!");
            eprintln!("  stdout: {}", stdout);
            eprintln!("  stderr: {}", stderr);
            eprintln!("Attempting fallback to Chocolatey...");

            // Try Chocolatey as fallback
            println!("=== DEBUG: Installing exiftool via Chocolatey (fallback) ===");

            // Ensure Chocolatey is installed
            if let Err(_) = windows::ensure_chocolatey_installed() {
                msi_progress::report_action_data("Chocolatey failed, trying bundled installer...");
                println!("Chocolatey installation also failed. Attempting bundled installer fallback...");

                // Try bundled installer as final fallback
                match install_from_bundled("exiftool", "windows") {
                    Ok(()) => {
                        println!("ExifTool installed successfully from bundled installer!");
                        msi_progress::report_action_data("ExifTool installed from bundled fallback");
                        report_progress("ExifTool installed (bundled fallback)", 25);
                        return Ok(()); // Exit early - dependency installed successfully
                    }
                    Err(bundle_err) => {
                        msi_progress::report_action_data("ERROR: All installation methods failed");
                        return Err(format!(
                            "\n╔══════════════════════════════════════════════════════════════════╗\n\
                             ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                             ╚══════════════════════════════════════════════════════════════════╝\n\n\
                             ExifTool is required for Nameback to function.\n\n\
                             All installation methods failed:\n\
                             • Scoop: {}\n\
                             • Chocolatey: Failed to install Chocolatey\n\
                             • Bundled installer: {}\n\n\
                             Please install manually:\n\
                             • Download from: https://exiftool.org/\n\
                             • Or run: choco install exiftool -y\n", stdout, bundle_err
                        ));
                    }
                }
            }

            // Try installing exiftool via Chocolatey
            match windows::install_package_via_chocolatey("exiftool") {
                Ok((choco_stdout, _choco_stderr)) => {
                    // Installation succeeded
                }
                Err(_) => {
                    msi_progress::report_action_data("ERROR: Chocolatey installation also failed");
                    return Err(format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         ExifTool is required for Nameback to function.\n\n\
                         Both Scoop and Chocolatey failed to install exiftool.\n\n\
                         Scoop error:\n{}\n\n\
                         Please install manually:\n\
                         • Download from: https://exiftool.org/\n", stdout
                    ));
                }
            }
        }

        msi_progress::report_action_data("exiftool installed successfully");
        println!("exiftool installed successfully");

        report_progress("Installing tesseract (optional OCR support)...", 60);
        msi_progress::report_action_data("Downloading and installing tesseract (optional)...");
        println!("=== DEBUG: Installing tesseract (optional) ===");
        println!("Full command: {} /c \"{}\" install tesseract", cmd_exe, scoop_cmd);

        let tesseract_result = Command::new(&cmd_exe)
            .arg("/c")
            .arg(&scoop_cmd)
            .arg("install")
            .arg("tesseract")
            .output();

        match tesseract_result {
            Ok(output) => {
                println!("tesseract install exit code: {:?}", output.status.code());
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("tesseract install stdout: {}", stdout);
                println!("tesseract install stderr: {}", stderr);

                // Check for errors in stdout (Scoop reports errors there)
                let has_error = !output.status.success() ||
                                stdout.contains("Failed to extract") ||
                                stdout.contains("is not valid") ||
                                stdout.contains("ERROR") ||
                                stdout.contains("could not be resolved");

                if has_error {
                    msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
                    println!("WARNING: tesseract installation via Scoop failed!");
                    println!("  stdout: {}", stdout);
                    println!("  stderr: {}", stderr);
                    println!("Attempting fallback to Chocolatey...");

                    // Try Chocolatey as fallback
                    println!("=== DEBUG: Installing tesseract via Chocolatey (fallback) ===");

                    // Ensure Chocolatey is installed
                    if let Err(_) = windows::ensure_chocolatey_installed() {
                        msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (tesseract is optional)");
                        println!("WARNING: Both Scoop and Chocolatey installation attempts failed for tesseract");
                        println!("  Tesseract is optional - only needed for OCR support");
                        println!("  You can install it manually later: choco install tesseract -y");
                    } else {
                        // Try installing tesseract via Chocolatey
                        match windows::install_package_via_chocolatey("tesseract") {
                            Ok(_) => {
                                // Installation succeeded
                            }
                            Err(_) => {
                                msi_progress::report_action_data("WARNING: tesseract installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for tesseract");
                                println!("  Tesseract is optional - only needed for OCR support");
                            }
                        }
                    }
                } else {
                    msi_progress::report_action_data("tesseract installed successfully");
                    println!("tesseract installed successfully");
                }
            }
            Err(e) => {
                msi_progress::report_action_data("WARNING: tesseract install command failed");
                println!("WARNING: Failed to execute tesseract install command: {}", e);
            }
        }

        report_progress("Installing ffmpeg (optional video support)...", 80);
        msi_progress::report_action_data("Downloading and installing ffmpeg (optional)...");
        println!("=== DEBUG: Installing ffmpeg (optional) ===");
        println!("Full command: {} /c \"{}\" install ffmpeg", cmd_exe, scoop_cmd);

        let ffmpeg_result = Command::new(&cmd_exe)
            .arg("/c")
            .arg(&scoop_cmd)
            .arg("install")
            .arg("ffmpeg")
            .output();

        match ffmpeg_result {
            Ok(output) => {
                println!("ffmpeg install exit code: {:?}", output.status.code());
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("ffmpeg install stdout: {}", stdout);
                println!("ffmpeg install stderr: {}", stderr);

                // Check for errors in stdout (Scoop reports errors there)
                let has_error = !output.status.success() ||
                                stdout.contains("Failed to extract") ||
                                stdout.contains("is not valid") ||
                                stdout.contains("ERROR") ||
                                stdout.contains("could not be resolved");

                if has_error {
                    msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
                    println!("WARNING: ffmpeg installation via Scoop failed!");
                    println!("  stdout: {}", stdout);
                    println!("  stderr: {}", stderr);
                    println!("Attempting fallback to Chocolatey...");

                    // Try Chocolatey as fallback
                    println!("=== DEBUG: Installing ffmpeg via Chocolatey (fallback) ===");

                    // Ensure Chocolatey is installed
                    if let Err(_) = windows::ensure_chocolatey_installed() {
                        msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (ffmpeg is optional)");
                        println!("WARNING: Both Scoop and Chocolatey installation attempts failed for ffmpeg");
                        println!("  FFmpeg is optional - only needed for video frame extraction");
                        println!("  You can install it manually later: choco install ffmpeg -y");
                    } else {
                        // Try installing ffmpeg via Chocolatey
                        match windows::install_package_via_chocolatey("ffmpeg") {
                            Ok(_) => {
                                // Installation succeeded
                            }
                            Err(_) => {
                                msi_progress::report_action_data("WARNING: ffmpeg installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for ffmpeg");
                                println!("  FFmpeg is optional - only needed for video frame extraction");
                            }
                        }
                    }
                } else {
                    msi_progress::report_action_data("ffmpeg installed successfully");
                    println!("ffmpeg installed successfully");
                }
            }
            Err(e) => {
                msi_progress::report_action_data("WARNING: ffmpeg install command failed");
                println!("WARNING: Failed to execute ffmpeg install command: {}", e);
            }
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        msi_progress::report_action_data("Downloading and installing imagemagick (optional)...");
        println!("=== DEBUG: Installing imagemagick (optional) ===");
        println!("Full command: {} /c \"{}\" install imagemagick", cmd_exe, scoop_cmd);

        let imagemagick_result = Command::new(&cmd_exe)
            .arg("/c")
            .arg(&scoop_cmd)
            .arg("install")
            .arg("imagemagick")
            .output();

        match imagemagick_result {
            Ok(output) => {
                println!("imagemagick install exit code: {:?}", output.status.code());
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("imagemagick install stdout: {}", stdout);
                println!("imagemagick install stderr: {}", stderr);

                // Check for errors in stdout (Scoop reports errors there)
                let has_error = !output.status.success() ||
                                stdout.contains("Failed to extract") ||
                                stdout.contains("is not valid") ||
                                stdout.contains("ERROR") ||
                                stdout.contains("could not be resolved");

                if has_error {
                    msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
                    println!("WARNING: imagemagick installation via Scoop failed!");
                    println!("  stdout: {}", stdout);
                    println!("  stderr: {}", stderr);
                    println!("Attempting fallback to Chocolatey...");

                    // Try Chocolatey as fallback
                    println!("=== DEBUG: Installing imagemagick via Chocolatey (fallback) ===");

                    // Ensure Chocolatey is installed
                    if let Err(_) = windows::ensure_chocolatey_installed() {
                        msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (imagemagick is optional)");
                        println!("WARNING: Both Scoop and Chocolatey installation attempts failed for imagemagick");
                        println!("  ImageMagick is optional - only needed for HEIC/HEIF image support");
                        println!("  You can install it manually later: choco install imagemagick -y");
                    } else {
                        // Try installing imagemagick via Chocolatey
                        match windows::install_package_via_chocolatey("imagemagick") {
                            Ok(_) => {
                                // Installation succeeded
                            }
                            Err(_) => {
                                msi_progress::report_action_data("WARNING: imagemagick installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for imagemagick");
                                println!("  ImageMagick is optional - only needed for HEIC/HEIF image support");
                            }
                        }
                    }
                } else {
                    msi_progress::report_action_data("imagemagick installed successfully");
                    println!("imagemagick installed successfully");
                }
            }
            Err(e) => {
                msi_progress::report_action_data("WARNING: imagemagick install command failed");
                println!("WARNING: Failed to execute imagemagick install command: {}", e);
            }
        }

        report_progress("Windows dependencies installed", 100);
        msi_progress::report_action_data("Dependency installation complete");

        // Ensure DNS is restored even if we didn't explicitly restore it earlier
        restore_dns();
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
