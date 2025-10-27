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

        // Get USERPROFILE path for Scoop installation location
        let user_profile = std::env::var("USERPROFILE")
            .map_err(|_| "USERPROFILE environment variable not set".to_string())?;
        let scoop_path = format!("{}\\scoop\\shims\\scoop.cmd", user_profile);

        // Check if Scoop is installed
        let scoop_check = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
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
                .arg("-WindowStyle")
                .arg("Hidden")
                .arg("-ExecutionPolicy")
                .arg("Bypass")
                .arg("-Command")
                .arg("iex (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')")
                .output()
                .map_err(|e| format!("Failed to install Scoop: {}", e))?;

            if !scoop_install.status.success() {
                let stderr = String::from_utf8_lossy(&scoop_install.stderr);
                log::error!("Scoop installation failed: {}", stderr);
                return Err(format!("Failed to install Scoop: {}. Please install it manually from https://scoop.sh", stderr));
            }

            log::info!("Scoop installed successfully to {}", user_profile);
        }

        // Use full path to scoop after installation
        report_progress("Installing exiftool (required)...", 40);
        let exiftool_result = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(format!("& '{}' install exiftool", scoop_path))
            .output()
            .map_err(|e| format!("Failed to run scoop install exiftool: {}", e))?;

        if !exiftool_result.status.success() {
            let stderr = String::from_utf8_lossy(&exiftool_result.stderr);
            let stdout = String::from_utf8_lossy(&exiftool_result.stdout);
            log::error!("exiftool installation failed. stdout: {}, stderr: {}", stdout, stderr);
            return Err(format!("Failed to install exiftool: {}", stderr));
        }

        log::info!("exiftool installed successfully");

        report_progress("Installing tesseract (optional OCR support)...", 60);
        let tesseract_result = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(format!("& '{}' install tesseract", scoop_path))
            .output();

        match tesseract_result {
            Ok(output) if output.status.success() => {
                log::info!("tesseract installed successfully");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Failed to install tesseract: {}", stderr);
            }
            Err(e) => {
                log::warn!("Failed to run scoop install tesseract: {}", e);
            }
        }

        report_progress("Installing ffmpeg (optional video support)...", 80);
        let ffmpeg_result = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(format!("& '{}' install ffmpeg", scoop_path))
            .output();

        match ffmpeg_result {
            Ok(output) if output.status.success() => {
                log::info!("ffmpeg installed successfully");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Failed to install ffmpeg: {}", stderr);
            }
            Err(e) => {
                log::warn!("Failed to run scoop install ffmpeg: {}", e);
            }
        }

        report_progress("Installing imagemagick (optional HEIC support)...", 90);
        let imagemagick_result = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(format!("& '{}' install imagemagick", scoop_path))
            .output();

        match imagemagick_result {
            Ok(output) if output.status.success() => {
                log::info!("imagemagick installed successfully");
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::warn!("Failed to install imagemagick: {}", stderr);
            }
            Err(e) => {
                log::warn!("Failed to run scoop install imagemagick: {}", e);
            }
        }

        report_progress("Windows dependencies installed", 100);
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
