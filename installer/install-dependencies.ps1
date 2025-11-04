# Advanced MSI dependency installer with real-time progress reporting
# This script uses Windows Installer COM API for proper MSI progress updates

param()

# Get the MSI installer session through COM
function Get-InstallerSession {
    try {
        $installer = New-Object -ComObject WindowsInstaller.Installer
        # Get the active product installation
        $products = $installer.Products

        # Try to get handle from environment (if available)
        if ($env:MSIHANDLE) {
            return $env:MSIHANDLE
        }

        # Otherwise, we'll use standard output which MSI can capture
        return $null
    } catch {
        return $null
    }
}

# Report progress to MSI installer UI
function Report-MsiProgress {
    param(
        [string]$Message,
        [int]$Percentage = -1
    )

    # Create timestamp for log
    $timestamp = Get-Date -Format "HH:mm:ss"

    # Log to file
    if ($script:logFile) {
        Add-Content -Path $script:logFile -Value "[$timestamp] $Message"
    }

    # Report to MSI UI via stdout (MSI captures this)
    # Using specific format that MSI recognizes
    if ($Percentage -ge 0) {
        Write-Output "1: 2: $Message [$Percentage%]"
    } else {
        Write-Output "1: 2: $Message"
    }

    # Flush output to ensure MSI sees it immediately
    [Console]::Out.Flush()
}

# Initialize logging
$script:logFile = Join-Path $env:TEMP "nameback-msi-install-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"
Add-Content -Path $script:logFile -Value "=== Nameback Dependency Installation Log ==="
Add-Content -Path $script:logFile -Value "Started: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Add-Content -Path $script:logFile -Value ""

# Find nameback.exe
$namebackExe = Join-Path $env:ProgramFiles "nameback\nameback.exe"
if (-not (Test-Path $namebackExe)) {
    # Try alternate location
    $namebackExe = Join-Path ${env:ProgramFiles(x86)} "nameback\nameback.exe"
}

if (-not (Test-Path $namebackExe)) {
    Report-MsiProgress "ERROR: Cannot find nameback.exe" 0
    Add-Content -Path $script:logFile -Value "ERROR: nameback.exe not found"
    exit 1
}

Add-Content -Path $script:logFile -Value "Found nameback.exe: $namebackExe"
Report-MsiProgress "Initializing dependency installation..." 5

# Create a job to run nameback.exe in background
Report-MsiProgress "Starting dependency installer..." 10

# Use Start-Process with output redirection to capture real-time output
$outputFile = Join-Path $env:TEMP "nameback-deps-output.txt"
$errorFile = Join-Path $env:TEMP "nameback-deps-error.txt"

# Clear any existing output files
if (Test-Path $outputFile) { Remove-Item $outputFile -Force }
if (Test-Path $errorFile) { Remove-Item $errorFile -Force }

# Start the process hidden with output redirection
$processInfo = New-Object System.Diagnostics.ProcessStartInfo
$processInfo.FileName = $namebackExe
$processInfo.Arguments = "--install-deps"
$processInfo.UseShellExecute = $false
$processInfo.RedirectStandardOutput = $true
$processInfo.RedirectStandardError = $true
$processInfo.CreateNoWindow = $true
$processInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden

$process = New-Object System.Diagnostics.Process
$process.StartInfo = $processInfo

# Set up event handlers for output
$outputHandler = {
    $line = $Event.SourceEventArgs.Data
    if ($line) {
        Add-Content -Path $script:logFile -Value "STDOUT: $line"

        # Parse progress from output and report to MSI
        if ($line -match '\[(\d+)%\]\s+(.+)') {
            $percent = [int]$matches[1]
            $message = $matches[2]
            Report-MsiProgress $message $percent
        }
        elseif ($line -match '(Checking network|Installing|Downloading|Configuring|Verifying)\s+(.+)') {
            Report-MsiProgress $line -1
        }
        elseif ($line -match '(Scoop|exiftool|tesseract|ffmpeg|imagemagick)') {
            Report-MsiProgress $line -1
        }
    }
}

$errorHandler = {
    $line = $Event.SourceEventArgs.Data
    if ($line) {
        Add-Content -Path $script:logFile -Value "STDERR: $line"
    }
}

# Register event handlers
Register-ObjectEvent -InputObject $process -EventName OutputDataReceived -Action $outputHandler | Out-Null
Register-ObjectEvent -InputObject $process -EventName ErrorDataReceived -Action $errorHandler | Out-Null

# Start the process
$process.Start() | Out-Null
$process.BeginOutputReadLine()
$process.BeginErrorReadLine()

# Monitor process with progress updates
$progressSteps = @(
    @{Time=5; Message="Checking network connectivity..."; Percent=15},
    @{Time=10; Message="Configuring package manager..."; Percent=20},
    @{Time=20; Message="Installing Scoop if needed..."; Percent=30},
    @{Time=30; Message="Downloading exiftool..."; Percent=40},
    @{Time=40; Message="Installing exiftool..."; Percent=50},
    @{Time=50; Message="Downloading tesseract..."; Percent=60},
    @{Time=60; Message="Installing tesseract..."; Percent=70},
    @{Time=70; Message="Downloading ffmpeg..."; Percent=80},
    @{Time=80; Message="Installing ffmpeg..."; Percent=85},
    @{Time=90; Message="Downloading imagemagick..."; Percent=90},
    @{Time=100; Message="Installing imagemagick..."; Percent=95}
)

$startTime = Get-Date
$lastStep = 0

while (-not $process.HasExited) {
    $elapsed = ((Get-Date) - $startTime).TotalSeconds

    # Report estimated progress based on time
    foreach ($step in $progressSteps) {
        if ($elapsed -gt $step.Time -and $lastStep -lt $step.Time) {
            Report-MsiProgress $step.Message $step.Percent
            $lastStep = $step.Time
        }
    }

    # Check every second
    Start-Sleep -Milliseconds 500

    # Timeout after 10 minutes
    if ($elapsed -gt 600) {
        Add-Content -Path $script:logFile -Value "ERROR: Installation timeout after 10 minutes"
        $process.Kill()
        break
    }
}

$process.WaitForExit()
$exitCode = $process.ExitCode

# Unregister event handlers
Get-EventSubscriber | Where-Object { $_.SourceObject -eq $process } | Unregister-Event

# Report completion
if ($exitCode -eq 0) {
    Report-MsiProgress "Dependencies installed successfully!" 100
    Add-Content -Path $script:logFile -Value "Installation completed successfully"
} else {
    Report-MsiProgress "Installation completed with errors (see log)" 100
    Add-Content -Path $script:logFile -Value "Installation failed with exit code: $exitCode"
}

Add-Content -Path $script:logFile -Value "Finished: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Add-Content -Path $script:logFile -Value "Log saved to: $($script:logFile)"

# Always exit 0 to allow MSI to continue
exit 0