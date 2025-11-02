//! DNS fallback helpers for macOS
//!
//! Provides functionality to temporarily switch to public DNS servers
//! when network connectivity issues prevent package manager downloads.

use std::process::Command;

/// Attempts to switch to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)
///
/// Saves the original DNS settings to the NAMEBACK_ORIGINAL_DNS_MACOS environment variable
/// for later restoration.
///
/// # Returns
/// * `Ok(())` if DNS was switched successfully
/// * `Err(String)` if the DNS switch failed
pub fn try_with_public_dns() -> Result<(), String> {
    println!("\n=== DNS FALLBACK (macOS): Attempting to use public DNS servers ===");

    // Get list of active network services
    let services_output = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .map_err(|e| format!("Failed to list network services: {}", e))?;

    if !services_output.status.success() {
        return Err("Failed to list network services".to_string());
    }

    let services = String::from_utf8_lossy(&services_output.stdout);
    let active_services: Vec<&str> = services
        .lines()
        .skip(1) // Skip the asterisk line
        .filter(|line| !line.starts_with('*'))
        .collect();

    // Save original DNS settings for each service
    let mut original_dns = Vec::new();
    for service in &active_services {
        if let Ok(output) = Command::new("networksetup")
            .args(["-getdnsservers", service])
            .output()
        {
            let dns = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("Saved DNS for {}: {}", service, dns);
            original_dns.push((service.to_string(), dns));
        }
    }

    // Store original DNS in environment variable for restoration
    let dns_json = serde_json::to_string(&original_dns)
        .map_err(|e| format!("Failed to serialize DNS settings: {}", e))?;
    std::env::set_var("NAMEBACK_ORIGINAL_DNS_MACOS", &dns_json);

    // Set public DNS for all services
    for service in &active_services {
        println!("Setting public DNS for {}...", service);
        let _ = Command::new("networksetup")
            .args(["-setdnsservers", service, "8.8.8.8", "8.8.4.4", "1.1.1.1"])
            .status();
    }

    println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
    Ok(())
}

/// Restores original DNS settings from the NAMEBACK_ORIGINAL_DNS_MACOS environment variable
///
/// This function should be called after operations that required public DNS are complete.
/// It safely restores the original DNS configuration and cleans up the environment variable.
pub fn restore_dns() {
    if let Ok(dns_json) = std::env::var("NAMEBACK_ORIGINAL_DNS_MACOS") {
        println!("\n=== DNS FALLBACK (macOS): Restoring original DNS settings ===");

        if let Ok(original_dns) = serde_json::from_str::<Vec<(String, String)>>(&dns_json) {
            for (service, dns) in original_dns {
                println!("Restoring DNS for {}: {}", service, dns);
                if dns.contains("There aren't any DNS Servers") || dns.is_empty() {
                    // Reset to DHCP
                    let _ = Command::new("networksetup")
                        .args(["-setdnsservers", &service, "empty"])
                        .status();
                } else {
                    // Restore specific DNS servers
                    let servers: Vec<&str> = dns.split('\n').collect();
                    let mut cmd = Command::new("networksetup");
                    cmd.arg("-setdnsservers").arg(&service);
                    for server in servers {
                        if !server.trim().is_empty() {
                            cmd.arg(server.trim());
                        }
                    }
                    let _ = cmd.status();
                }
            }
        }

        println!("DNS settings restored");
        std::env::remove_var("NAMEBACK_ORIGINAL_DNS_MACOS");
    }
}
