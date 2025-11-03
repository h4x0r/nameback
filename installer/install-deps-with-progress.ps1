# PowerShell wrapper for dependency installation with real-time MSI progress reporting
# This script monitors the log file output and reports progress to MSI UI

param(
    [string]$InstallFolder
)

# MSI progress reporting function
function Report-Progress {
    param([string]$Message)

    # For MSI progress reporting, we need to use the installer's progress mechanism
    # When running as a custom action, we can use Write-Host which MSI captures
    Write-Host $Message

    # Also try to report via MSI record if available
    try {
        $record = New-Object -ComObject WindowsInstaller.Installer
        # This won't work directly, but keeping for reference
    } catch {
        # Silent fail - Write-Host will be captured by MSI
    }
}

$namebackExe = Join-Path $InstallFolder "nameback.exe"

# Check if executable exists
if (-not (Test-Path $namebackExe)) {
    Report-Progress "ERROR: nameback.exe not found"
    exit 0
}

Report-Progress "Starting dependency installation..."
Report-Progress "This may take several minutes..."

# Run the executable with real-time output capture
$process = Start-Process -FilePath $namebackExe `
                        -ArgumentList "--install-deps" `
                        -NoNewWindow `
                        -PassThru `
                        -RedirectStandardOutput (Join-Path $env:TEMP "nameback-install-stdout.log") `
                        -RedirectStandardError (Join-Path $env:TEMP "nameback-install-stderr.log")

# Monitor the log file for progress updates
$logFile = Get-ChildItem "$env:TEMP\nameback-install-*.log" -ErrorAction SilentlyContinue |
           Sort-Object LastWriteTime -Descending |
           Select-Object -First 1

$lastPosition = 0
$checkInterval = 1  # Check every second

while (-not $process.HasExited) {
    Start-Sleep -Seconds $checkInterval

    # Check for new log file if we don't have one yet
    if (-not $logFile -or -not (Test-Path $logFile.FullName)) {
        $logFile = Get-ChildItem "$env:TEMP\nameback-install-*.log" -ErrorAction SilentlyContinue |
                   Sort-Object LastWriteTime -Descending |
                   Select-Object -First 1
        continue
    }

    # Read new content from log file
    try {
        $content = Get-Content $logFile.FullName -ErrorAction SilentlyContinue
        if ($content -and $content.Count -gt $lastPosition) {
            $newLines = $content[$lastPosition..($content.Count - 1)]
            foreach ($line in $newLines) {
                # Extract progress messages (lines with percentage or key actions)
                if ($line -match '\[(\d+)%\](.+)' -or
                    $line -match '(Installing|Downloading|Checking|Configuring)' -or
                    $line -match '(Scoop|exiftool|tesseract|ffmpeg|imagemagick)') {

                    # Clean up the message
                    $cleanLine = $line -replace '\[\d+:\d+:\d+\]', '' -replace '\[INFO\]', '' -replace '\[WARN\]', ''
                    $cleanLine = $cleanLine.Trim()

                    if ($cleanLine) {
                        Report-Progress $cleanLine
                    }
                }
            }
            $lastPosition = $content.Count
        }
    } catch {
        # Continue if we can't read the file
    }
}

$process.WaitForExit()

# Report completion
if ($process.ExitCode -eq 0) {
    Report-Progress "Dependencies installed successfully"
} else {
    Report-Progress "Installation completed with exit code: $($process.ExitCode)"
}

# Read final output
if (Test-Path (Join-Path $env:TEMP "nameback-install-stdout.log")) {
    $stdout = Get-Content (Join-Path $env:TEMP "nameback-install-stdout.log") -Raw
    if ($stdout) {
        Report-Progress "=== Final Output ==="
        Report-Progress $stdout
    }
}

exit 0
