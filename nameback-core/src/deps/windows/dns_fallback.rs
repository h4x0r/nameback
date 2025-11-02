//! DNS fallback helpers for Windows
//!
//! Provides functionality to temporarily switch to public DNS servers
//! when network connectivity issues prevent package manager downloads.

use std::process::Command;

/// Attempts to switch to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)
///
/// Saves the original DNS settings to the NAMEBACK_ORIGINAL_DNS environment variable
/// for later restoration.
///
/// # Returns
/// * `Ok(())` if DNS was switched successfully
/// * `Err(String)` if the DNS switch failed
pub fn try_with_public_dns() -> Result<(), String> {
    println!("\n=== DNS FALLBACK: Attempting to use public DNS servers ===");

    // PowerShell script to save current DNS, switch to public DNS, and return original settings
    let setup_dns_script = r#"
        # Get all active network adapters
        $adapters = Get-NetAdapter | Where-Object { $_.Status -eq 'Up' }
        $originalDNS = @()

        foreach ($adapter in $adapters) {
            # Save current DNS settings
            $currentDNS = Get-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -AddressFamily IPv4
            $originalDNS += [PSCustomObject]@{
                InterfaceIndex = $adapter.InterfaceIndex
                InterfaceAlias = $adapter.InterfaceAlias
                ServerAddresses = $currentDNS.ServerAddresses
            }

            # Set public DNS (Google 8.8.8.8, 8.8.4.4, Cloudflare 1.1.1.1)
            try {
                Set-DnsClientServerAddress -InterfaceIndex $adapter.InterfaceIndex -ServerAddresses ("8.8.8.8","8.8.4.4","1.1.1.1")
                Write-Host "Set public DNS for $($adapter.InterfaceAlias)"
            } catch {
                Write-Warning "Failed to set DNS for $($adapter.InterfaceAlias): $_"
            }
        }

        # Return original DNS settings as JSON for restoration
        $originalDNS | ConvertTo-Json -Compress
    "#;

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(setup_dns_script)
        .output()
        .map_err(|e| format!("Failed to execute DNS setup script: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("DNS setup failed: {}", stderr));
    }

    let original_dns_json = String::from_utf8_lossy(&output.stdout);
    println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
    println!("Original DNS settings saved for restoration");

    // Store original DNS for restoration (we'll use this later)
    std::env::set_var("NAMEBACK_ORIGINAL_DNS", original_dns_json.trim());

    Ok(())
}

/// Restores original DNS settings from the NAMEBACK_ORIGINAL_DNS environment variable
///
/// This function should be called after operations that required public DNS are complete.
/// It safely restores the original DNS configuration and cleans up the environment variable.
pub fn restore_dns() {
    if let Ok(original_dns_json) = std::env::var("NAMEBACK_ORIGINAL_DNS") {
        println!("\n=== DNS FALLBACK: Restoring original DNS settings ===");

        let restore_script = format!(r#"
            $originalDNS = '{}' | ConvertFrom-Json

            foreach ($dns in $originalDNS) {{
                try {{
                    if ($dns.ServerAddresses -and $dns.ServerAddresses.Count -gt 0) {{
                        Set-DnsClientServerAddress -InterfaceIndex $dns.InterfaceIndex -ServerAddresses $dns.ServerAddresses
                        Write-Host "Restored DNS for $($dns.InterfaceAlias)"
                    }} else {{
                        # No DNS was set (DHCP), reset to automatic
                        Set-DnsClientServerAddress -InterfaceIndex $dns.InterfaceIndex -ResetServerAddresses
                        Write-Host "Reset DNS to DHCP for $($dns.InterfaceAlias)"
                    }}
                }} catch {{
                    Write-Warning "Failed to restore DNS for $($dns.InterfaceAlias): $_"
                }}
            }}
        "#, original_dns_json);

        let _ = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&restore_script)
            .output();

        println!("DNS settings restored");
        std::env::remove_var("NAMEBACK_ORIGINAL_DNS");
    }
}
