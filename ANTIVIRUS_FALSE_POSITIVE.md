# Windows Defender False Positive - Trojan:Win32/Wacatac.B!ml

## Issue

Windows Defender may quarantine `nameback.exe` with the detection name **Trojan:Win32/Wacatac.B!ml**. This is a **false positive** caused by heuristic analysis of legitimate functionality.

## Why This Happens

The CLI binary includes Windows API calls for console window management (lines 6-30 in nameback-cli/src/main.rs):

```rust
#[cfg(windows)]
fn hide_console_if_msi() {
    // Hide console window during MSI installation to prevent flash
    if let Ok(msihandle) = std::env::var("MSIHANDLE") {
        if msihandle.parse::<u32>().is_ok() {
            use windows::Win32::System::Console::GetConsoleWindow;
            use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

            unsafe {
                let console_window = GetConsoleWindow();
                if !console_window.0.is_null() {
                    ShowWindow(console_window, SW_HIDE);
                }
            }
        }
    }
}
```

**Why this triggers detection:**
- Uses `unsafe` Windows API calls (GetConsoleWindow, ShowWindow)
- Manipulates window visibility
- Rust binaries have unusual binary patterns compared to MSVC-compiled code
- Heuristic engines flag "suspicious" behavior patterns

**This is legitimate functionality** needed to prevent console window flashing during MSI installation.

## Verification That This Is Safe

You can verify the source code:
1. Visit the GitHub repository: https://github.com/h4x0r/nameback
2. Review the code in `nameback-cli/src/main.rs` lines 6-30
3. Check that the binary matches the published source (reproducible builds)
4. Verify the SHA256 checksum matches the GitHub release

## Solutions

### For End Users

**Option 1: Restore from quarantine and add exclusion**
1. Open Windows Security
2. Go to "Virus & threat protection" → "Protection history"
3. Find the quarantined `nameback.exe`
4. Click "Actions" → "Restore"
5. Add exclusion: "Virus & threat protection" → "Manage settings" → "Exclusions" → "Add or remove exclusions"
6. Add the folder: `C:\Program Files\nameback\`

**Option 2: Use the GUI instead**
The GUI executable (`nameback-gui.exe`) does not include this console manipulation code and should not trigger false positives.

**Option 3: Install via Chocolatey** (when available)
Packages in the official Chocolatey repository undergo additional verification.

### For Developers

**Option 1: Submit to Microsoft for analysis**

1. Go to: https://www.microsoft.com/en-us/wdsi/filesubmission
2. Submit `nameback.exe` for analysis
3. Select "Software developer" as your organization type
4. Explain the legitimate use of Windows API calls
5. Provide source code link: https://github.com/h4x0r/nameback

**Option 2: Code signing**

Sign the binary with an EV (Extended Validation) code signing certificate:
- Costs $300-$500/year
- Requires business verification
- Significantly reduces false positives
- Builds trust with Windows SmartScreen

**Option 3: Add version information resource**

Add a Windows resource file with version metadata:
- Company name
- Product name
- File description
- Copyright information

This helps antivirus engines recognize it as legitimate software.

**Option 4: Alternative implementation**

Instead of hiding the console window, consider:
- Running the MSI custom actions as GUI subsystem processes
- Using PowerShell wrappers that don't require console manipulation
- Deferring dependency checks to post-installation GUI flow

## Current Status

- **Reported to Microsoft:** ⏳ Pending (submit at link above)
- **Code signing:** ⏳ Not yet implemented (requires certificate purchase)
- **Workaround:** ✅ Documented (add Windows Defender exclusion)

## Related Issues

- GitHub Issue: #TBD (create an issue to track this)
- Microsoft Submission: #TBD (after submission)

## References

- Source code: nameback-cli/src/main.rs:6-30
- Windows API documentation: https://docs.microsoft.com/en-us/windows/console/getconsolewindow
- Similar issues: Many Rust projects encounter this (ripgrep, bat, fd, etc.)
