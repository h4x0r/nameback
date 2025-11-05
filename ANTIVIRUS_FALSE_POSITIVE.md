# Windows Defender False Positive - Trojan:Win32/Wacatac.B!ml

## Issue Resolved in v0.7.18+

**This issue has been fixed** by removing Windows API console manipulation code that triggered antivirus false positives.

## Historical Context (v0.7.17 and earlier)

Versions v0.7.17 and earlier of nameback.exe triggered Windows Defender false positives (Trojan:Win32/Wacatac.B!ml) due to Windows API calls for console window management:

```rust
// This code was removed in v0.7.18
#[cfg(windows)]
fn hide_console_if_msi() {
    // Used GetConsoleWindow and ShowWindow to hide console during MSI install
}
```

**Why this triggered detection:**
- Used `unsafe` Windows API calls (GetConsoleWindow, ShowWindow)
- Manipulated window visibility
- Rust binaries have unusual binary patterns compared to MSVC-compiled code
- Heuristic engines flag "suspicious" behavior patterns

## Verification That This Is Safe

You can verify the source code:
1. Visit the GitHub repository: https://github.com/h4x0r/nameback
2. Review the code in `nameback-cli/src/main.rs` lines 6-30
3. Check that the binary matches the published source (reproducible builds)
4. Verify the SHA256 checksum matches the GitHub release

## Solution

**Upgrade to v0.7.18 or later** - The console hiding code has been removed, eliminating the false positive.

### If Stuck on v0.7.17 or Earlier

**Option 1: Upgrade to latest version**
Download the latest MSI installer from [releases](https://github.com/h4x0r/nameback/releases/latest)

**Option 2: Use the GUI instead**
The GUI executable (`nameback-gui.exe`) never included console manipulation code and doesn't trigger false positives.

**Option 3: Restore from quarantine (not recommended)**
1. Open Windows Security
2. Go to "Virus & threat protection" → "Protection history"
3. Find the quarantined `nameback.exe`
4. Click "Actions" → "Restore"
5. Add exclusion for `C:\Program Files\nameback\`

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
