# Add Scoop shims directory to system PATH
# This script is called by the MSI installer after dependency installation

$ErrorActionPreference = 'Continue'

try {
    $userProfile = $env:USERPROFILE
    $scoopShims = Join-Path $userProfile 'scoop\shims'

    Write-Host "Checking for Scoop shims at: $scoopShims"

    if (Test-Path $scoopShims) {
        Write-Host "Scoop shims directory found"

        $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')

        if ($currentPath -notlike "*$scoopShims*") {
            Write-Host "Adding Scoop shims to system PATH..."
            $newPath = "$currentPath;$scoopShims"
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
            Write-Host "Successfully added Scoop shims to system PATH"
            Write-Host "PATH entry: $scoopShims"
        } else {
            Write-Host "Scoop shims already in system PATH"
        }
    } else {
        Write-Host "Scoop shims directory not found - dependencies may have been installed via Chocolatey instead"
    }

    # Also check for ImageMagick app directory (Scoop may not create shims for it)
    $imageMagickDir = Join-Path $userProfile 'scoop\apps\imagemagick\current'

    if (Test-Path $imageMagickDir) {
        Write-Host "Checking ImageMagick directory: $imageMagickDir"

        $currentPath = [Environment]::GetEnvironmentVariable('Path', 'Machine')

        if ($currentPath -notlike "*$imageMagickDir*") {
            Write-Host "Adding ImageMagick directory to system PATH..."
            $newPath = "$currentPath;$imageMagickDir"
            [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')
            Write-Host "Successfully added ImageMagick to system PATH"
            Write-Host "PATH entry: $imageMagickDir"
        } else {
            Write-Host "ImageMagick already in system PATH"
        }
    }

    Write-Host "PATH configuration completed"
    exit 0
} catch {
    Write-Host "ERROR: Failed to configure PATH: $_"
    Write-Host "This is non-fatal - dependencies may still work"
    exit 0  # Don't fail the installation
}
