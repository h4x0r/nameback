# MSI dependency installer with proper UI progress reporting
# This script reports progress to the MSI installer UI using ActionData

param()

# Initialize logging
$script:logFile = Join-Path $env:TEMP "nameback-msi-install-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"
Add-Content -Path $script:logFile -Value "=== Nameback Dependency Installation Log ==="
Add-Content -Path $script:logFile -Value "Started: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Add-Content -Path $script:logFile -Value ""

# Report progress to MSI UI
# MSI captures ActionData messages and displays them in [1] placeholder
function Report-Progress {
    param(
        [string]$Message
    )

    $timestamp = Get-Date -Format "HH:mm:ss"

    # Log to file
    Add-Content -Path $script:logFile -Value "[$timestamp] $Message"

    # Send ActionData message for MSI progress dialog
    # Format: "1: 2: message" for ActionData
    # This replaces [1] in ProgressText template
    Write-Host "ActionData: $Message"
    [Console]::Out.Flush()
}

# Find nameback.exe
$namebackExe = Join-Path $env:ProgramFiles "nameback\nameback.exe"
if (-not (Test-Path $namebackExe)) {
    $namebackExe = Join-Path ${env:ProgramFiles(x86)} "nameback\nameback.exe"
}

if (-not (Test-Path $namebackExe)) {
    Report-Progress "ERROR: Cannot find nameback.exe"
    exit 1
}

Report-Progress "Found nameback.exe at $namebackExe"
Report-Progress "Initializing dependency installation..."

# Start the process with output capture
$processInfo = New-Object System.Diagnostics.ProcessStartInfo
$processInfo.FileName = $namebackExe
$processInfo.Arguments = "--install-deps"
$processInfo.UseShellExecute = $false
$processInfo.RedirectStandardOutput = $true
$processInfo.RedirectStandardError = $true
$processInfo.CreateNoWindow = $true

$process = New-Object System.Diagnostics.Process
$process.StartInfo = $processInfo

try {
    $process.Start() | Out-Null

    Report-Progress "Starting dependency installer..."

    # Read output line by line and report to MSI
    while (-not $process.StandardOutput.EndOfStream) {
        $line = $process.StandardOutput.ReadLine()
        if ($line) {
            # Extract and report specific dependency names
            if ($line -match 'Downloading\s+(\w+)') {
                Report-Progress "Downloading $($matches[1])..."
            }
            elseif ($line -match 'Installing\s+(\w+)') {
                Report-Progress "Installing $($matches[1])..."
            }
            elseif ($line -match 'exiftool') {
                Report-Progress "Installing ExifTool (metadata extraction)..."
            }
            elseif ($line -match 'tesseract') {
                Report-Progress "Installing Tesseract (OCR engine)..."
            }
            elseif ($line -match 'ffmpeg') {
                Report-Progress "Installing FFmpeg (video processing)..."
            }
            elseif ($line -match 'imagemagick') {
                Report-Progress "Installing ImageMagick (image conversion)..."
            }
            elseif ($line -match 'Checking network') {
                Report-Progress "Checking network connectivity..."
            }
            elseif ($line -match 'Configuring') {
                Report-Progress "Configuring package manager..."
            }
            elseif ($line -match 'Successfully|completed') {
                Report-Progress "Finalizing installation..."
            }
            elseif ($line -match '(Error|Failed)') {
                Report-Progress $line
            }

            # Always log everything to file
            Add-Content -Path $script:logFile -Value $line
        }
    }

    # Also capture stderr
    $stderr = $process.StandardError.ReadToEnd()
    if ($stderr) {
        Add-Content -Path $script:logFile -Value "STDERR:"
        Add-Content -Path $script:logFile -Value $stderr
    }

    $process.WaitForExit()
    $exitCode = $process.ExitCode

    if ($exitCode -eq 0) {
        Report-Progress "All dependencies installed successfully!"
    } else {
        Report-Progress "Installation completed with errors (exit code: $exitCode)"
    }

} catch {
    $errorMsg = $_.Exception.Message
    Report-Progress "ERROR: $errorMsg"
    Add-Content -Path $script:logFile -Value "ERROR: $errorMsg"
    exit 1
}

Add-Content -Path $script:logFile -Value ""
Add-Content -Path $script:logFile -Value "Finished: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Add-Content -Path $script:logFile -Value "Log saved to: $script:logFile"

# Always return success to allow MSI to continue
# (dependencies are optional, installation should not fail if they fail)
exit 0
