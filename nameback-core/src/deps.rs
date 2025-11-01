use std::process::Command;

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
    let report_progress = |msg: &str, pct: u8| {
        // Send to MSI installer UI (Windows only)
        msi_progress::report_action_data(msg);

        // Also send to callback or stdout
        if let Some(ref cb) = progress {
            cb(msg, pct);
        } else {
            if pct == 0 {
                println!("\n==================================================");
                println!("  Installing Dependencies");
                println!("==================================================\n");
            }
            println!("{}", msg);
        }
    };

    msi_progress::report_action_start("Installing nameback dependencies");

    // Print version information at the start
    println!("=== NAMEBACK DEPENDENCY INSTALLER ===");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("======================================\n");

    report_progress("Starting installation...", 0);

    #[cfg(target_os = "windows")]
    {
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
                "Invoke-WebRequest -Uri 'https://get.scoop.sh' -OutFile '{}' -UseBasicParsing",
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

            // Check if Chocolatey is installed
            // Refresh PATH first in case it was just installed by another process
            let choco_check = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); Get-Command choco -ErrorAction SilentlyContinue")
                .output();

            let choco_installed = choco_check
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !choco_installed {
                println!("Chocolatey not found, installing...");
                msi_progress::report_action_data("Installing Chocolatey package manager...");

                // Install Chocolatey
                let choco_install = Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-Command")
                    .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))")
                    .output();

                match choco_install {
                    Ok(output) if output.status.success() => {
                        println!("Chocolatey installed successfully");
                        msi_progress::report_action_data("Chocolatey installed");
                    }
                    _ => {
                        msi_progress::report_action_data("ERROR: Both Scoop and Chocolatey failed");
                        return Err(format!(
                            "\n╔══════════════════════════════════════════════════════════════════╗\n\
                             ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                             ╚══════════════════════════════════════════════════════════════════╝\n\n\
                             ExifTool is required for Nameback to function.\n\n\
                             Both Scoop and Chocolatey installation attempts failed.\n\n\
                             Scoop error:\n{}\n\n\
                             Please install manually:\n\
                             • Download from: https://exiftool.org/\n\
                             • Or run: choco install exiftool -y\n", stdout
                        ));
                    }
                }
            }

            // Try installing exiftool via Chocolatey
            println!("Installing exiftool via Chocolatey...");
            msi_progress::report_action_data("Installing exiftool via Chocolatey...");

            // Chocolatey is installed to C:\ProgramData\chocolatey by default
            // After installation, we need to either:
            // 1. Use the full path to choco.exe
            // 2. Refresh the environment in PowerShell
            // We'll use approach #2 with environment refresh
            let choco_exiftool = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); choco install exiftool -y --no-progress")
                .output();

            match choco_exiftool {
                Ok(output) => {
                    let choco_stdout = String::from_utf8_lossy(&output.stdout);
                    let choco_stderr = String::from_utf8_lossy(&output.stderr);
                    println!("Chocolatey exiftool stdout: {}", choco_stdout);
                    println!("Chocolatey exiftool stderr: {}", choco_stderr);

                    if output.status.success() && !choco_stdout.contains("ERROR") {
                        println!("exiftool installed successfully via Chocolatey");
                        msi_progress::report_action_data("exiftool installed via Chocolatey");
                    } else {
                        msi_progress::report_action_data("ERROR: Chocolatey installation also failed");
                        return Err(format!(
                            "\n╔══════════════════════════════════════════════════════════════════╗\n\
                             ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                             ╚══════════════════════════════════════════════════════════════════╝\n\n\
                             ExifTool is required for Nameback to function.\n\n\
                             Both Scoop and Chocolatey failed to install exiftool.\n\n\
                             Scoop error:\n{}\n\n\
                             Chocolatey error:\n{}\n\n\
                             Please install manually:\n\
                             • Download from: https://exiftool.org/\n", stdout, choco_stdout
                        ));
                    }
                }
                Err(e) => {
                    msi_progress::report_action_data("ERROR: Failed to execute Chocolatey");
                    return Err(format!(
                        "\n╔══════════════════════════════════════════════════════════════════╗\n\
                         ║  EXIFTOOL INSTALLATION FAILED                                    ║\n\
                         ╚══════════════════════════════════════════════════════════════════╝\n\n\
                         ExifTool is required for Nameback to function.\n\n\
                         Scoop failed and Chocolatey could not be executed.\n\n\
                         Error: {}\n\n\
                         Please install manually:\n\
                         • Download from: https://exiftool.org/\n", e
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

                    // Check if Chocolatey is installed
                    let choco_check = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); Get-Command choco -ErrorAction SilentlyContinue")
                        .output();

                    let choco_installed = choco_check
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if !choco_installed {
                        println!("Chocolatey not found, installing...");
                        msi_progress::report_action_data("Installing Chocolatey package manager...");

                        let choco_install = Command::new("powershell")
                            .arg("-NoProfile")
                            .arg("-ExecutionPolicy")
                            .arg("Bypass")
                            .arg("-Command")
                            .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))")
                            .output();

                        match choco_install {
                            Ok(output) if output.status.success() => {
                                println!("Chocolatey installed successfully");
                                msi_progress::report_action_data("Chocolatey installed");
                            }
                            _ => {
                                msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (tesseract is optional)");
                                println!("WARNING: Both Scoop and Chocolatey installation attempts failed for tesseract");
                                println!("  Tesseract is optional - only needed for OCR support");
                                println!("  You can install it manually later: choco install tesseract -y");
                            }
                        }
                    }

                    // Try installing tesseract via Chocolatey
                    println!("Installing tesseract via Chocolatey...");
                    msi_progress::report_action_data("Installing tesseract via Chocolatey...");

                    let choco_tesseract = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-ExecutionPolicy")
                        .arg("Bypass")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); choco install tesseract -y --no-progress")
                        .output();

                    match choco_tesseract {
                        Ok(output) => {
                            let choco_stdout = String::from_utf8_lossy(&output.stdout);
                            let choco_stderr = String::from_utf8_lossy(&output.stderr);
                            println!("Chocolatey tesseract stdout: {}", choco_stdout);
                            println!("Chocolatey tesseract stderr: {}", choco_stderr);

                            if output.status.success() && !choco_stdout.contains("ERROR") {
                                println!("tesseract installed successfully via Chocolatey");
                                msi_progress::report_action_data("tesseract installed via Chocolatey");
                            } else {
                                msi_progress::report_action_data("WARNING: tesseract installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for tesseract");
                                println!("  Tesseract is optional - only needed for OCR support");
                            }
                        }
                        Err(e) => {
                            msi_progress::report_action_data("WARNING: Failed to execute Chocolatey for tesseract");
                            println!("WARNING: Failed to execute Chocolatey command: {}", e);
                            println!("  Tesseract is optional - only needed for OCR support");
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

                    // Check if Chocolatey is installed
                    let choco_check = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); Get-Command choco -ErrorAction SilentlyContinue")
                        .output();

                    let choco_installed = choco_check
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if !choco_installed {
                        println!("Chocolatey not found, installing...");
                        msi_progress::report_action_data("Installing Chocolatey package manager...");

                        let choco_install = Command::new("powershell")
                            .arg("-NoProfile")
                            .arg("-ExecutionPolicy")
                            .arg("Bypass")
                            .arg("-Command")
                            .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))")
                            .output();

                        match choco_install {
                            Ok(output) if output.status.success() => {
                                println!("Chocolatey installed successfully");
                                msi_progress::report_action_data("Chocolatey installed");
                            }
                            _ => {
                                msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (ffmpeg is optional)");
                                println!("WARNING: Both Scoop and Chocolatey installation attempts failed for ffmpeg");
                                println!("  FFmpeg is optional - only needed for video frame extraction");
                                println!("  You can install it manually later: choco install ffmpeg -y");
                            }
                        }
                    }

                    // Try installing ffmpeg via Chocolatey
                    println!("Installing ffmpeg via Chocolatey...");
                    msi_progress::report_action_data("Installing ffmpeg via Chocolatey...");

                    let choco_ffmpeg = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-ExecutionPolicy")
                        .arg("Bypass")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); choco install ffmpeg -y --no-progress")
                        .output();

                    match choco_ffmpeg {
                        Ok(output) => {
                            let choco_stdout = String::from_utf8_lossy(&output.stdout);
                            let choco_stderr = String::from_utf8_lossy(&output.stderr);
                            println!("Chocolatey ffmpeg stdout: {}", choco_stdout);
                            println!("Chocolatey ffmpeg stderr: {}", choco_stderr);

                            if output.status.success() && !choco_stdout.contains("ERROR") {
                                println!("ffmpeg installed successfully via Chocolatey");
                                msi_progress::report_action_data("ffmpeg installed via Chocolatey");
                            } else {
                                msi_progress::report_action_data("WARNING: ffmpeg installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for ffmpeg");
                                println!("  FFmpeg is optional - only needed for video frame extraction");
                            }
                        }
                        Err(e) => {
                            msi_progress::report_action_data("WARNING: Failed to execute Chocolatey for ffmpeg");
                            println!("WARNING: Failed to execute Chocolatey command: {}", e);
                            println!("  FFmpeg is optional - only needed for video frame extraction");
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

                    // Check if Chocolatey is installed
                    // Refresh PATH first in case it was just installed
                    let choco_check = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); Get-Command choco -ErrorAction SilentlyContinue")
                        .output();

                    let choco_installed = choco_check
                        .map(|o| o.status.success())
                        .unwrap_or(false);

                    if !choco_installed {
                        println!("Chocolatey not found, installing...");
                        msi_progress::report_action_data("Installing Chocolatey package manager...");

                        // Install Chocolatey
                        let choco_install = Command::new("powershell")
                            .arg("-NoProfile")
                            .arg("-ExecutionPolicy")
                            .arg("Bypass")
                            .arg("-Command")
                            .arg("[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))")
                            .output();

                        match choco_install {
                            Ok(output) if output.status.success() => {
                                println!("Chocolatey installed successfully");
                                msi_progress::report_action_data("Chocolatey installed");
                            }
                            _ => {
                                msi_progress::report_action_data("WARNING: Both Scoop and Chocolatey failed (imagemagick is optional)");
                                println!("WARNING: Both Scoop and Chocolatey installation attempts failed for imagemagick");
                                println!("  ImageMagick is optional - only needed for HEIC/HEIF image support");
                                println!("  You can install it manually later: choco install imagemagick -y");
                            }
                        }
                    }

                    // Try installing imagemagick via Chocolatey
                    println!("Installing imagemagick via Chocolatey...");
                    msi_progress::report_action_data("Installing imagemagick via Chocolatey...");

                    let choco_imagemagick = Command::new("powershell")
                        .arg("-NoProfile")
                        .arg("-ExecutionPolicy")
                        .arg("Bypass")
                        .arg("-Command")
                        .arg("$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); choco install imagemagick -y --no-progress")
                        .output();

                    match choco_imagemagick {
                        Ok(output) => {
                            let choco_stdout = String::from_utf8_lossy(&output.stdout);
                            let choco_stderr = String::from_utf8_lossy(&output.stderr);
                            println!("Chocolatey imagemagick stdout: {}", choco_stdout);
                            println!("Chocolatey imagemagick stderr: {}", choco_stderr);

                            if output.status.success() && !choco_stdout.contains("ERROR") {
                                println!("imagemagick installed successfully via Chocolatey");
                                msi_progress::report_action_data("imagemagick installed via Chocolatey");
                            } else {
                                msi_progress::report_action_data("WARNING: imagemagick installation failed (optional)");
                                println!("WARNING: Chocolatey installation also failed for imagemagick");
                                println!("  ImageMagick is optional - only needed for HEIC/HEIF image support");
                            }
                        }
                        Err(e) => {
                            msi_progress::report_action_data("WARNING: Failed to execute Chocolatey for imagemagick");
                            println!("WARNING: Failed to execute Chocolatey command: {}", e);
                            println!("  ImageMagick is optional - only needed for HEIC/HEIF image support");
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
    }

    #[cfg(target_os = "macos")]
    {
        report_progress("Checking Homebrew installation...", 10);

        // Check if Homebrew is installed
        let brew_check = Command::new("brew")
            .arg("--version")
            .output();

        let brew_installed = brew_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !brew_installed {
            return Err("Homebrew is not installed. Please install from https://brew.sh".to_string());
        }

        report_progress("Installing exiftool (required)...", 30);
        let exiftool_status = Command::new("brew")
            .args(&["install", "exiftool"])
            .status()
            .map_err(|e| format!("Failed to install exiftool: {}", e))?;

        if !exiftool_status.success() {
            return Err("Failed to install exiftool".to_string());
        }

        report_progress("Installing tesseract (optional OCR support)...", 50);
        if let Err(e) = Command::new("brew")
            .args(&["install", "tesseract", "tesseract-lang"])
            .status()
        {
            log::warn!("Failed to install tesseract: {}", e);
        }

        report_progress("Installing ffmpeg (optional video support)...", 70);
        if let Err(e) = Command::new("brew")
            .args(&["install", "ffmpeg"])
            .status()
        {
            log::warn!("Failed to install ffmpeg: {}", e);
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        if let Err(e) = Command::new("brew")
            .args(&["install", "imagemagick"])
            .status()
        {
            log::warn!("Failed to install imagemagick: {}", e);
        }

        report_progress("macOS dependencies installed", 100);
    }

    #[cfg(target_os = "linux")]
    {
        report_progress("Detecting package manager...", 10);

        // Detect which package manager is available
        let (pkg_manager, install_cmd) = if Command::new("apt-get").arg("--version").output().is_ok() {
            ("apt-get", vec!["install", "-y"])
        } else if Command::new("dnf").arg("--version").output().is_ok() {
            ("dnf", vec!["install", "-y"])
        } else if Command::new("yum").arg("--version").output().is_ok() {
            ("yum", vec!["install", "-y"])
        } else if Command::new("pacman").arg("--version").output().is_ok() {
            ("pacman", vec!["-S", "--noconfirm"])
        } else {
            return Err("No supported package manager found (apt-get, dnf, yum, or pacman required)".to_string());
        };

        report_progress(&format!("Using {} package manager...", pkg_manager), 20);

        // Check if running with sudo/root
        let needs_sudo = std::env::var("USER").unwrap_or_default() != "root";

        report_progress("Installing exiftool (required)...", 30);
        let mut exiftool_cmd = if needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg(pkg_manager);
            cmd
        } else {
            Command::new(pkg_manager)
        };

        for arg in &install_cmd {
            exiftool_cmd.arg(arg);
        }
        exiftool_cmd.arg(if pkg_manager == "pacman" { "perl-image-exiftool" } else { "libimage-exiftool-perl" });

        let exiftool_status = exiftool_cmd
            .status()
            .map_err(|e| format!("Failed to install exiftool: {}", e))?;

        if !exiftool_status.success() {
            return Err("Failed to install exiftool. You may need to run with sudo.".to_string());
        }

        report_progress("Installing tesseract (optional OCR support)...", 50);
        let mut tesseract_cmd = if needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg(pkg_manager);
            cmd
        } else {
            Command::new(pkg_manager)
        };
        for arg in &install_cmd {
            tesseract_cmd.arg(arg);
        }
        tesseract_cmd.arg("tesseract-ocr");
        if let Err(e) = tesseract_cmd.status() {
            log::warn!("Failed to install tesseract: {}", e);
        }

        report_progress("Installing ffmpeg (optional video support)...", 70);
        let mut ffmpeg_cmd = if needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg(pkg_manager);
            cmd
        } else {
            Command::new(pkg_manager)
        };
        for arg in &install_cmd {
            ffmpeg_cmd.arg(arg);
        }
        ffmpeg_cmd.arg("ffmpeg");
        if let Err(e) = ffmpeg_cmd.status() {
            log::warn!("Failed to install ffmpeg: {}", e);
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        let mut imagemagick_cmd = if needs_sudo {
            let mut cmd = Command::new("sudo");
            cmd.arg(pkg_manager);
            cmd
        } else {
            Command::new(pkg_manager)
        };
        for arg in &install_cmd {
            imagemagick_cmd.arg(arg);
        }
        imagemagick_cmd.arg("imagemagick");
        if let Err(e) = imagemagick_cmd.status() {
            log::warn!("Failed to install imagemagick: {}", e);
        }

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
