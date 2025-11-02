# Refactoring Opportunities

This document identifies code quality improvements and refactoring opportunities in the Nameback codebase.

## Priority 1: Critical Refactoring (Do First)

### 1. ⚠️ deps.rs is Too Large (1,861 lines)

**Problem:**
- Single file contains all platform-specific installation logic
- Windows, macOS, Linux code all interleaved
- Hard to navigate and maintain
- Violates Single Responsibility Principle

**Solution:** Split into module structure

**Proposed Structure:**
```
nameback-core/src/deps/
├── mod.rs                      # Public API, shared types
├── common.rs                   # is_command_available, check_dependencies
├── bundled.rs                  # install_from_bundled logic
├── windows/
│   ├── mod.rs                  # Windows entry point
│   ├── scoop.rs                # Scoop installation
│   ├── chocolatey.rs           # Chocolatey installation
│   ├── dns_fallback.rs         # DNS switching logic
│   └── msi_progress.rs         # MSI progress reporting
├── macos/
│   ├── mod.rs                  # macOS entry point
│   ├── homebrew.rs             # Homebrew installation
│   ├── macports.rs             # MacPorts fallback
│   └── dns_fallback.rs         # DNS switching logic
└── linux/
    ├── mod.rs                  # Linux entry point
    ├── apt.rs                  # apt/dpkg
    ├── dnf.rs                  # dnf/rpm
    ├── pacman.rs               # Arch Linux
    └── dns_fallback.rs         # DNS switching logic
```

**Benefits:**
- Each file < 300 lines
- Clear separation by platform
- Easier to test individual components
- Can add new package managers easily
- Better for team collaboration

**Estimated Effort:** 4-6 hours

---

### 2. 🔄 Massive Code Duplication in deps.rs

**Problem:**
The exiftool, tesseract, ffmpeg, and imagemagick installation sections have near-identical code:

```rust
// This pattern repeats 4 times with minor variations:
if has_error {
    msi_progress::report_action_data("Scoop failed, trying Chocolatey fallback...");
    // Check if Chocolatey is installed
    let choco_check = Command::new("powershell")...
    // Install Chocolatey if needed
    // Try installing via Chocolatey
    // Try bundled installer fallback
    // Show error message
}
```

**Solution:** Extract to shared functions

```rust
// In deps/windows/common.rs
fn install_with_scoop_or_fallback(
    package_name: &str,
    scoop_name: &str,
    choco_name: &str,
    display_name: &str,
) -> Result<(), String> {
    // Layer 1: Try Scoop
    if try_scoop_install(scoop_name).is_ok() {
        return Ok(());
    }

    // Layer 2: DNS fallback + retry Scoop
    if try_with_dns_fallback(|| try_scoop_install(scoop_name)).is_ok() {
        return Ok(());
    }

    // Layer 3: Try Chocolatey
    if try_chocolatey_install(choco_name).is_ok() {
        return Ok(());
    }

    // Layer 4: Bundled installer
    if install_from_bundled(package_name, "windows").is_ok() {
        return Ok(());
    }

    // Layer 5: Error
    Err(format_installation_error(display_name))
}
```

**Usage:**
```rust
install_with_scoop_or_fallback("exiftool", "exiftool", "exiftool", "ExifTool")?;
install_with_scoop_or_fallback("tesseract", "tesseract", "tesseract", "Tesseract")?;
install_with_scoop_or_fallback("ffmpeg", "ffmpeg", "ffmpeg", "FFmpeg")?;
```

**Benefits:**
- Reduces 1,200+ lines to ~300 lines
- Single place to fix bugs
- Consistent behavior across all dependencies
- Easier to add Layer 6, 7, etc.

**Estimated Effort:** 3-4 hours

---

### 3. 🧪 Dead Code and Unused Variables

**Problem:** Clippy reports multiple warnings:

```rust
// deps.rs:192 - unused because not all code paths use it yet
warning: unused variable: `install_from_bundled`

// deps_check.rs:64
warning: variable does not need to be mutable
let mut needs_imagemagick = false;

// lib.rs:112
warning: value assigned to `analyses` is never read
let mut analyses = Vec::new();

// lib.rs:392
warning: method `analyze_file` is never used
fn analyze_file(...) { }

// extractor.rs:157
warning: field `creator` is never read
struct ExiftoolOutput {
    creator: Option<String>,  // Never used
}
```

**Solution:** Clean up dead code

```bash
# Auto-fix most issues
cargo clippy --fix --lib -p nameback-core

# Manually review and remove:
# - analyze_file method (dead code)
# - creator field (unused)
# - analyses variable assignment
```

**Benefits:**
- Cleaner codebase
- No misleading code
- Better compile times
- Clearer intent

**Estimated Effort:** 30 minutes

---

## Priority 2: Code Quality Improvements

### 4. 🔒 Error Handling Could Be Better

**Problem:** Mixing error types and string formatting

```rust
// Current
return Err(format!("Failed to download: {}", e));
return Err("Failed to run extraction command".to_string());

// Mixing Result<(), String> with anyhow::Result
```

**Solution:** Use proper error types

```rust
// Create custom error type
#[derive(Debug, thiserror::Error)]
pub enum DepsError {
    #[error("Package manager failed: {0}")]
    PackageManagerFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(#[from] reqwest::Error),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("All installation methods failed")]
    AllMethodsFailed {
        scoop: Option<String>,
        chocolatey: Option<String>,
        bundled: Option<String>,
    },
}

// Then use
pub fn run_installer() -> Result<(), DepsError> { ... }
```

**Benefits:**
- Structured error information
- Better error messages
- Type-safe error handling
- Can add context easily

**Estimated Effort:** 2-3 hours

---

### 5. 📝 DNS Fallback Logic is Duplicated

**Problem:** DNS fallback appears 3 times (once per platform) with similar but slightly different code.

**Solution:** Extract to shared trait

```rust
// In deps/common.rs
pub trait DnsFallback {
    fn save_dns_settings(&self) -> Result<String, String>;
    fn apply_public_dns(&self) -> Result<(), String>;
    fn restore_dns_settings(&self, original: &str) -> Result<(), String>;
}

// Then implement per platform
impl DnsFallback for WindowsDns { ... }
impl DnsFallback for MacOsDns { ... }
impl DnsFallback for LinuxDns { ... }

// Generic retry with DNS fallback
pub fn retry_with_dns_fallback<F, T>(
    operation: F,
    dns: &dyn DnsFallback,
) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(e) if is_dns_error(&e) => {
            let original = dns.save_dns_settings()?;
            dns.apply_public_dns()?;
            let result = operation();
            dns.restore_dns_settings(&original)?;
            result
        }
        Err(e) => Err(e),
    }
}
```

**Benefits:**
- Single implementation of retry logic
- Platform-specific DNS handling
- Testable in isolation
- Guaranteed cleanup

**Estimated Effort:** 2 hours

---

### 6. 🧩 Bundled Installer Has Platform-Specific Code Inside

**Problem:** The `install_from_bundled` closure has nested `#[cfg]` blocks making it hard to read.

**Solution:** Extract platform-specific installers

```rust
// In deps/bundled.rs
pub fn install_from_bundled(dep_name: &str, platform: &str) -> Result<(), String> {
    let installer_path = download_from_github(dep_name, platform)?;

    #[cfg(target_os = "windows")]
    return windows::install_bundled(&installer_path, dep_name);

    #[cfg(target_os = "macos")]
    return macos::install_bundled(&installer_path, dep_name);

    #[cfg(target_os = "linux")]
    return linux::install_bundled(&installer_path, dep_name);
}

// Then in deps/windows/bundled.rs
pub fn install_bundled(installer_path: &Path, dep_name: &str) -> Result<(), String> {
    let extract_dir = extract_archive(installer_path)?;

    match dep_name {
        "tesseract" => install_tesseract_setup(&extract_dir),
        _ => install_portable(&extract_dir, dep_name),
    }
}
```

**Benefits:**
- Clear separation per platform
- Each platform file < 100 lines
- Easier to add new dependencies
- Can test each platform independently

**Estimated Effort:** 1-2 hours

---

## Priority 3: Nice-to-Have Improvements

### 7. 📊 Progress Reporting is Scattered

**Problem:** Progress reporting mixed throughout installation logic

```rust
msi_progress::report_action_data("Installing exiftool...");
report_progress("ExifTool installed", 25);
println!("Installing exiftool via Scoop...");
```

**Solution:** Centralized progress reporting

```rust
pub struct ProgressReporter {
    callback: Option<ProgressCallback>,
    msi_enabled: bool,
}

impl ProgressReporter {
    pub fn report(&self, message: &str, percentage: u8) {
        if self.msi_enabled {
            msi_progress::report_action_data(message);
        }
        if let Some(ref cb) = self.callback {
            cb(message, percentage);
        } else {
            println!("[{}%] {}", percentage, message);
        }
    }
}
```

**Benefits:**
- Single source of truth
- Consistent formatting
- Easier to add new output methods
- Testable

**Estimated Effort:** 1 hour

---

### 8. 🧪 Missing Tests for New Code

**Problem:** The bundled installer fallback has no tests

**Solution:** Add integration tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_download_from_github_success() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/releases/download/v0.7.0/deps-exiftool-windows.zip")
            .with_status(200)
            .with_body(b"fake zip content")
            .create();

        let result = download_from_github("exiftool", "windows");
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_download_from_github_not_found() {
        let mut server = Server::new();
        let mock = server.mock("GET", "/releases/download/v0.7.0/deps-fake-windows.zip")
            .with_status(404)
            .create();

        let result = download_from_github("fake", "windows");
        assert!(result.is_err());
    }
}
```

**Benefits:**
- Confidence in bundled fallback
- Catch regressions early
- Document expected behavior

**Estimated Effort:** 2 hours

---

### 9. 🔧 Clippy Suggestions

**Problem:** 13 clippy warnings for simple improvements

```rust
// Using contains() instead of iter().any()
warning: using `contains()` instead of `iter().any()` is more efficient

// Stripping prefix manually
warning: stripping a prefix manually

// Complex map_or
warning: this `map_or` can be simplified

// Confusing method name
warning: method `default` can be confused for the standard trait method
```

**Solution:** Auto-fix most

```bash
cargo clippy --fix --lib -p nameback-core
```

**Benefits:**
- More idiomatic Rust
- Better performance
- Clearer code

**Estimated Effort:** 15 minutes (automated)

---

### 10. 📦 Constants Should Be Centralized

**Problem:** URLs and version numbers scattered throughout

```rust
// In multiple places:
"https://github.com/h4x0r/nameback/releases/download/v{}/{}",
"https://exiftool.org/exiftool-{}.zip"
"https://community.chocolatey.org/install.ps1"
```

**Solution:** Centralize constants

```rust
// deps/constants.rs
pub mod urls {
    pub const GITHUB_RELEASES: &str = "https://github.com/h4x0r/nameback/releases/download";
    pub const EXIFTOOL_DOWNLOAD: &str = "https://exiftool.org";
    pub const CHOCOLATEY_INSTALL: &str = "https://community.chocolatey.org/install.ps1";
}

pub mod versions {
    pub const EXIFTOOL: &str = "12.70";
    pub const TESSERACT: &str = "5.3.3";
    pub const FFMPEG: &str = "6.1";
}
```

**Benefits:**
- Single place to update URLs
- Easy to see what external dependencies exist
- Can validate URLs in tests

**Estimated Effort:** 30 minutes

---

## Summary Table

| Priority | Refactoring | Impact | Effort | LOC Reduction |
|----------|-------------|--------|--------|---------------|
| 1 | Split deps.rs into modules | High | 4-6h | N/A (reorganize) |
| 1 | Extract duplicate installation logic | High | 3-4h | -900 lines |
| 1 | Remove dead code | Medium | 30min | -50 lines |
| 2 | Better error handling | Medium | 2-3h | ~same |
| 2 | DNS fallback trait | Medium | 2h | -200 lines |
| 2 | Extract bundled platform logic | Medium | 1-2h | ~same |
| 3 | Centralized progress reporting | Low | 1h | -100 lines |
| 3 | Add tests for bundled fallback | Low | 2h | +200 lines |
| 3 | Apply clippy fixes | Low | 15min | -10 lines |
| 3 | Centralize constants | Low | 30min | ~same |

**Total Estimated Effort:** 16-21 hours
**Total LOC Reduction:** ~1,260 lines
**Final deps.rs Size:** ~600 lines (down from 1,861)

---

## Recommended Order

### Phase 1: Quick Wins (2 hours)
1. ✅ Run `cargo clippy --fix` (15 min)
2. ✅ Remove dead code (30 min)
3. ✅ Centralize constants (30 min)
4. ✅ Centralized progress reporting (1 hour)

### Phase 2: Core Refactoring (10 hours)
5. ✅ Extract duplicate installation logic (3-4 hours)
6. ✅ DNS fallback trait (2 hours)
7. ✅ Better error handling (2-3 hours)
8. ✅ Extract bundled platform logic (1-2 hours)

### Phase 3: Structural (6 hours)
9. ✅ Split deps.rs into modules (4-6 hours)

### Phase 4: Testing (2 hours)
10. ✅ Add tests for bundled fallback (2 hours)

---

## Benefits Summary

**After Full Refactoring:**
- ✅ deps.rs: 1,861 lines → ~600 lines (68% reduction)
- ✅ Much easier to add new package managers
- ✅ Clear separation of concerns
- ✅ Platform-specific code isolated
- ✅ Consistent error handling
- ✅ Comprehensive test coverage
- ✅ Easier for new contributors
- ✅ Less code duplication (DRY principle)

---

## Should You Refactor Now?

**Arguments FOR:**
- Code is fresh in mind
- Before next feature additions
- Before it gets worse
- Easier to maintain going forward

**Arguments AGAINST:**
- Current code works
- No immediate bugs
- Time investment needed
- Risk of introducing bugs

**Recommendation:**
Do **Phase 1 (Quick Wins)** immediately - low risk, high value.
Schedule **Phase 2-3** for next sprint/version.
