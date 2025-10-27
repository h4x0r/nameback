use std::process::Command;

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

    report_progress("Starting installation...", 0);

    #[cfg(target_os = "windows")]
    {
        report_progress("Checking Scoop installation...", 10);

        // Check if Scoop is installed
        let scoop_check = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("Get-Command scoop -ErrorAction SilentlyContinue")
            .output();

        let scoop_installed = scoop_check
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !scoop_installed {
            report_progress("Installing Scoop package manager (no admin required)...", 20);

            // Install Scoop using the official installation command
            // Scoop installs to user directory, no admin/UAC needed
            let scoop_install = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg("iex (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')")
                .status()
                .map_err(|e| format!("Failed to install Scoop: {}", e))?;

            if !scoop_install.success() {
                return Err("Failed to install Scoop. Please install it manually from https://scoop.sh".to_string());
            }
        }

        report_progress("Installing exiftool (required)...", 40);
        let exiftool_status = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("scoop install exiftool")
            .status()
            .map_err(|e| format!("Failed to install exiftool: {}", e))?;

        if !exiftool_status.success() {
            return Err("Failed to install exiftool".to_string());
        }

        report_progress("Installing tesseract (optional OCR support)...", 60);
        let _ = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("scoop install tesseract")
            .status();

        report_progress("Installing ffmpeg (optional video support)...", 80);
        let _ = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("scoop install ffmpeg")
            .status();

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        let _ = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg("scoop install imagemagick")
            .status();

        report_progress("Windows dependencies installed", 100);
    }

    #[cfg(target_os = "macos")]
    {
        report_progress("Installing macOS dependencies...", 25);
        let status = Command::new("bash")
            .arg("install-deps-macos.sh")
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err("Installer script failed".to_string());
        }
        report_progress("macOS dependencies installed", 100);
    }

    #[cfg(target_os = "linux")]
    {
        report_progress("Installing Linux dependencies...", 25);
        let status = Command::new("bash")
            .arg("install-deps-linux.sh")
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err("Installer script failed".to_string());
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
