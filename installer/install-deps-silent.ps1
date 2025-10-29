# Silent dependency installer for MSI
# Runs nameback.exe --install-deps without showing console window

$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$exePath = Join-Path $scriptPath "nameback.exe"

# Start the process hidden and wait for it to complete
$process = Start-Process -FilePath $exePath -ArgumentList "--install-deps" -WindowStyle Hidden -PassThru -Wait

# Return the exit code
exit $process.ExitCode
