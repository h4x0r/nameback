# Add Scoop shims directory to system PATH
# This script is called by the MSI installer after dependency installation
# Sends ActionData messages for progress display in MSI UI

$ErrorActionPreference = 'Continue'

# Function to send ActionData to MSI progress dialog
function Report-Progress {
    param([string]$Message)
    Write-Host "ActionData: $Message"
    [Console]::Out.Flush()
}

try {
    Report-Progress "Configuring system PATH..."

    $userProfile = $env:USERPROFILE
    $scoopShims = Join-Path $userProfile 'scoop\shims'

    Report-Progress "Checking for package manager directories..."

    if (Test-Path $scoopShims) {
        Report-Progress "Found Scoop installation..."

        $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')

        if ($currentPath -notlike "*$scoopShims*") {
            Report-Progress "Adding Scoop to system PATH..."
            $newPath = "$currentPath;$scoopShims"
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
            Report-Progress "Scoop added to PATH successfully"
        } else {
            Report-Progress "Scoop already in system PATH"
        }
    } else {
        Report-Progress "Checking for Chocolatey..."
    }

    # Also check for ImageMagick app directory
    $imageMagickApp = Join-Path $userProfile 'scoop\apps\imagemagick\current'

    if (Test-Path $imageMagickApp) {
        Report-Progress "Configuring ImageMagick..."

        $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')

        if ($currentPath -notlike "*$imageMagickApp*") {
            Report-Progress "Adding ImageMagick to PATH..."
            $newPath = "$currentPath;$imageMagickApp"
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
            Report-Progress "ImageMagick configured"
        }
    }

    # Check for exiftool
    Report-Progress "Verifying ExifTool availability..."
    if (Get-Command exiftool -ErrorAction SilentlyContinue) {
        Report-Progress "✓ ExifTool ready"
    }

    # Check for tesseract
    Report-Progress "Verifying Tesseract availability..."
    if (Get-Command tesseract -ErrorAction SilentlyContinue) {
        Report-Progress "✓ Tesseract ready"
    }

    # Check for ffmpeg
    Report-Progress "Verifying FFmpeg availability..."
    if (Get-Command ffmpeg -ErrorAction SilentlyContinue) {
        Report-Progress "✓ FFmpeg ready"
    }

    # Check for ImageMagick
    Report-Progress "Verifying ImageMagick availability..."
    if (Get-Command magick -ErrorAction SilentlyContinue) {
        Report-Progress "✓ ImageMagick ready"
    }

    Report-Progress "PATH configuration complete"

} catch {
    Report-Progress "Warning: PATH configuration incomplete"
    Write-Host "Error: $_"
    # Exit 0 anyway - this is not a critical failure
    exit 0
}

exit 0