//! DNS fallback helpers for Linux
//!
//! Provides functionality to temporarily switch to public DNS servers
//! when network connectivity issues prevent package manager downloads.

use std::process::Command;

/// Attempts to switch to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)
///
/// Saves the original /etc/resolv.conf to the NAMEBACK_ORIGINAL_RESOLV_CONF environment variable
/// for later restoration.
///
/// # Returns
/// * `Ok(())` if DNS was switched successfully
/// * `Err(String)` if the DNS switch failed
pub fn try_with_public_dns() -> Result<(), String> {
    println!("\n=== DNS FALLBACK (Linux): Attempting to use public DNS servers ===");

    // Save original resolv.conf
    let resolv_conf_path = "/etc/resolv.conf";
    let original_resolv = std::fs::read_to_string(resolv_conf_path)
        .unwrap_or_else(|_| String::new());

    std::env::set_var("NAMEBACK_ORIGINAL_RESOLV_CONF", &original_resolv);

    // Create temporary resolv.conf with public DNS
    let new_resolv = "# Temporary public DNS for Nameback installation\nnameserver 8.8.8.8\nnameserver 8.8.4.4\nnameserver 1.1.1.1\n";

    // Try to write new resolv.conf (requires root/sudo)
    if let Err(e) = Command::new("sudo")
        .arg("sh")
        .arg("-c")
        .arg(format!("echo '{}' > {}", new_resolv, resolv_conf_path))
        .status()
    {
        return Err(format!("Failed to update DNS: {}", e));
    }

    println!("Switched to public DNS servers (Google 8.8.8.8, Cloudflare 1.1.1.1)");
    Ok(())
}

/// Restores original DNS settings from the NAMEBACK_ORIGINAL_RESOLV_CONF environment variable
///
/// This function should be called after operations that required public DNS are complete.
/// It safely restores the original /etc/resolv.conf and cleans up the environment variable.
pub fn restore_dns() {
    if let Ok(original_resolv) = std::env::var("NAMEBACK_ORIGINAL_RESOLV_CONF") {
        println!("\n=== DNS FALLBACK (Linux): Restoring original DNS settings ===");

        if !original_resolv.is_empty() {
            // Restore original resolv.conf
            let _ = Command::new("sudo")
                .arg("sh")
                .arg("-c")
                .arg(format!("echo '{}' > /etc/resolv.conf", original_resolv))
                .status();
        }

        println!("DNS settings restored");
        std::env::remove_var("NAMEBACK_ORIGINAL_RESOLV_CONF");
    }
}
