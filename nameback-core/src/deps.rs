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

/// Runs the appropriate installer script based on the platform
pub fn run_installer() -> Result<(), String> {
    println!("\n==================================================");
    println!("  Installing Dependencies");
    println!("==================================================\n");

    #[cfg(target_os = "windows")]
    {
        println!("Launching Windows dependency installer...\n");
        let status = Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg("install-deps-windows.ps1")
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err("Installer script failed".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("Launching macOS dependency installer...\n");
        let status = Command::new("bash")
            .arg("install-deps-macos.sh")
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err("Installer script failed".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("Launching Linux dependency installer...\n");
        let status = Command::new("bash")
            .arg("install-deps-linux.sh")
            .status()
            .map_err(|e| format!("Failed to run installer: {}", e))?;

        if !status.success() {
            return Err("Installer script failed".to_string());
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        return Err("Unsupported platform. Please install dependencies manually.".to_string());
    }

    println!("\n==================================================");
    println!("  Installation Complete!");
    println!("==================================================\n");

    Ok(())
}
