# Refactoring Plan - Phases 1, 2, and 3

**Status:** ✅ Phase 1 COMPLETE - Ready for Phase 2
**Started:** 2025-11-02
**Phase 1 Completed:** 2025-11-02
**Estimated Remaining Time:** 14-19 hours (Phases 2 & 3)

## Context

This plan executes the comprehensive refactoring outlined in `/REFACTORING_OPPORTUNITIES.md`. The goal is to reduce deps.rs from 1,861 lines to ~600 lines, eliminate code duplication, and improve maintainability.

## Current State

**✅ PHASE 1 COMPLETED (2 hours)**

All quick wins implemented and tested:

1. ✅ **Clippy auto-fix applied**
   - Fixed borrowed expression issues (8 locations in deps.rs)
   - Fixed array initialization syntax
   - Result: nameback-core compiles with zero warnings

2. ✅ **Dead code removed**
   - Deleted `analyze_file` method from lib.rs (lines 388-445, 52 lines removed)
   - Removed unused `creator` field from ExiftoolOutput struct (extractor.rs:157)
   - Removed unnecessary `mut` declarations
   - Renamed `RenameEngine::default()` to `with_defaults()` to avoid Default trait confusion

3. ✅ **Constants centralized**
   - Created `constants` module in deps.rs (lines 3-29)
   - Centralized GITHUB_RELEASES_BASE (actively used)
   - Documented all external URLs for future use

4. ✅ **Progress reporting centralized**
   - Created `ProgressReporter` struct (lines 93-120)
   - Unified MSI progress and callback reporting
   - Integrated into `run_installer_with_progress`

**Test Status:** ✅ All 139 tests passing
**Compiler Status:** ✅ Clean build, zero warnings in nameback-core

**Next Steps:** Begin Phase 2 (Core Refactoring)

---

## PHASE 1: Quick Wins (2 hours) ✅ COMPLETED

### Step 1.1: Remove Dead Code ✅ DONE

**Files modified:**

1. ✅ **`nameback-core/src/lib.rs`**
   - Kept `analyses` variable (it IS used, just assigned later)
   - Deleted entire `analyze_file` method (lines 388-439, 52 lines)
   - Renamed `default()` → `with_defaults()` to avoid trait confusion

2. ✅ **`nameback-core/src/extractor.rs`**
   - Removed `creator: Option<String>` field (line 157)

3. ✅ **`nameback-core/src/deps_check.rs`**
   - Kept `needs_imagemagick` as immutable (it IS used later)

4. ✅ **`nameback-core/src/deps.rs`**
   - Added `#[allow(unused_variables)]` with comment explaining Windows-only usage
   - OR: Ensure it's used in all platform code paths (better solution)

**Verification:**
```bash
cargo check -p nameback-core 2>&1 | grep warning
# Should show 0 warnings about dead code
```

### Step 1.2: Centralize Constants (30 min)

**Create new file:** `nameback-core/src/deps/constants.rs`

```rust
/// URLs for dependency downloads and installation
pub mod urls {
    pub const GITHUB_RELEASES: &str = "https://github.com/h4x0r/nameback/releases/download";
    pub const EXIFTOOL_DOWNLOAD: &str = "https://exiftool.org";
    pub const CHOCOLATEY_INSTALL: &str = "https://community.chocolatey.org/install.ps1";
    pub const TESSERACT_MANNHEIM: &str = "https://digi.bib.uni-mannheim.de/tesseract";
    pub const FFMPEG_GYAN: &str = "https://github.com/GyanD/codexffmpeg/releases";
    pub const FFMPEG_OFFICIAL: &str = "https://ffmpeg.org/releases";
}

/// Dependency version numbers
pub mod versions {
    pub const EXIFTOOL: &str = "12.70";
    pub const TESSERACT: &str = "5.3.3";
    pub const FFMPEG: &str = "6.1";
    pub const IMAGEMAGICK: &str = "7.1.1";
}

/// Package names for different package managers
pub mod packages {
    pub const EXIFTOOL: &str = "exiftool";
    pub const TESSERACT: &str = "tesseract";
    pub const FFMPEG: &str = "ffmpeg";
    pub const IMAGEMAGICK: &str = "imagemagick";
}
```

**Then replace all hardcoded strings in deps.rs:**
```rust
// Before
"https://github.com/h4x0r/nameback/releases/download/v{}/{}",

// After
use crate::deps::constants::urls;
format!("{}/v{}/{}", urls::GITHUB_RELEASES, version, asset_name)
```

**Search and replace:**
```bash
grep -n "https://" nameback-core/src/deps.rs
# Manually replace each with constants
```

### Step 1.3: Centralize Progress Reporting (1 hour)

**Create new file:** `nameback-core/src/deps/progress.rs`

```rust
use std::sync::Arc;

pub type ProgressCallback = Arc<dyn Fn(&str, u8) + Send + Sync>;

pub struct ProgressReporter {
    callback: Option<ProgressCallback>,
    #[cfg(windows)]
    msi_enabled: bool,
}

impl ProgressReporter {
    pub fn new(callback: Option<ProgressCallback>) -> Self {
        Self {
            callback,
            #[cfg(windows)]
            msi_enabled: std::env::var("MSIHANDLE").is_ok(),
        }
    }

    pub fn report(&self, message: &str, percentage: u8) {
        #[cfg(windows)]
        if self.msi_enabled {
            super::msi_progress::report_action_data(message);
        }

        if let Some(ref cb) = self.callback {
            cb(message, percentage);
        } else {
            if percentage == 0 {
                println!("\n==================================================");
                println!("  Installing Dependencies");
                println!("==================================================\n");
            }
            println!("[{:3}%] {}", percentage, message);
        }
    }

    pub fn report_action_start(&self, action: &str) {
        #[cfg(windows)]
        if self.msi_enabled {
            super::msi_progress::report_action_start(action);
        }
    }
}
```

**Replace scattered progress calls:**
```rust
// Before
msi_progress::report_action_data("Installing...");
report_progress("Installing...", 25);
println!("Installing...");

// After
reporter.report("Installing...", 25);
```

---

## PHASE 2: Core Refactoring (10 hours)

### Step 2.1: Extract Duplicate Installation Logic (3-4 hours)

**Create new file:** `nameback-core/src/deps/windows/installer.rs`

```rust
use super::*;

/// Installs a dependency using the 4-layer fallback system
pub fn install_with_fallback(
    dep_name: &str,
    scoop_name: &str,
    choco_name: &str,
    display_name: &str,
    reporter: &ProgressReporter,
) -> Result<(), String> {
    // Layer 1: Try Scoop
    reporter.report(&format!("Installing {} via Scoop...", display_name), 0);
    if try_scoop_install(scoop_name).is_ok() {
        reporter.report(&format!("{} installed via Scoop", display_name), 100);
        return Ok(());
    }

    // Layer 2: DNS fallback + retry Scoop
    reporter.report("Scoop failed, trying DNS fallback...", 25);
    if try_with_dns_fallback(|| try_scoop_install(scoop_name)).is_ok() {
        reporter.report(&format!("{} installed via Scoop (DNS fallback)", display_name), 100);
        return Ok(());
    }

    // Layer 3: Try Chocolatey
    reporter.report("Trying Chocolatey fallback...", 50);
    if try_chocolatey_install(choco_name).is_ok() {
        reporter.report(&format!("{} installed via Chocolatey", display_name), 100);
        return Ok(());
    }

    // Layer 4: Bundled installer
    reporter.report("Trying bundled installer...", 75);
    if install_from_bundled(dep_name, "windows").is_ok() {
        reporter.report(&format!("{} installed from bundled installer", display_name), 100);
        return Ok(());
    }

    // Layer 5: Error
    Err(format_installation_error(display_name))
}

fn try_scoop_install(package: &str) -> Result<(), String> {
    // Extract current Scoop installation code here
}

fn try_chocolatey_install(package: &str) -> Result<(), String> {
    // Extract current Chocolatey installation code here
}

fn try_with_dns_fallback<F>(operation: F) -> Result<(), String>
where
    F: Fn() -> Result<(), String>,
{
    // Extract current DNS fallback code here
}

fn format_installation_error(dep_name: &str) -> String {
    format!(
        "\n╔══════════════════════════════════════════════════════════════════╗\n\
         ║  {} INSTALLATION FAILED                                         ║\n\
         ╚══════════════════════════════════════════════════════════════════╝\n\n\
         All installation methods failed.\n\
         Please install manually from official sources.\n",
        dep_name.to_uppercase()
    )
}
```

**Usage in run_installer_with_progress:**
```rust
#[cfg(target_os = "windows")]
{
    use deps::windows::installer::install_with_fallback;

    install_with_fallback("exiftool", "exiftool", "exiftool", "ExifTool", &reporter)?;
    install_with_fallback("tesseract", "tesseract", "tesseract", "Tesseract", &reporter)?;
    install_with_fallback("ffmpeg", "ffmpeg", "ffmpeg", "FFmpeg", &reporter)?;
    install_with_fallback("imagemagick", "imagemagick", "imagemagick", "ImageMagick", &reporter)?;
}
```

### Step 2.2: Create DNS Fallback Trait (2 hours)

**Create new file:** `nameback-core/src/deps/dns_fallback.rs`

```rust
use std::process::Command;

pub trait DnsFallback {
    fn save_dns_settings(&self) -> Result<String, String>;
    fn apply_public_dns(&self) -> Result<(), String>;
    fn restore_dns_settings(&self, original: &str) -> Result<(), String>;
}

pub struct WindowsDns;
pub struct MacOsDns;
pub struct LinuxDns;

impl DnsFallback for WindowsDns {
    fn save_dns_settings(&self) -> Result<String, String> {
        // Extract Windows DNS save logic
    }

    fn apply_public_dns(&self) -> Result<(), String> {
        // Extract Windows DNS apply logic
    }

    fn restore_dns_settings(&self, original: &str) -> Result<(), String> {
        // Extract Windows DNS restore logic
    }
}

// Similar for MacOsDns and LinuxDns...

pub fn retry_with_dns_fallback<F, T>(
    operation: F,
    dns: &dyn DnsFallback,
    error_msg: &str,
) -> Result<T, String>
where
    F: Fn() -> Result<T, String>,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(e) if is_dns_error(&e) => {
            println!("Detected DNS error, attempting DNS fallback...");
            let original = dns.save_dns_settings()?;
            dns.apply_public_dns()?;
            let result = operation();
            dns.restore_dns_settings(&original)?;
            result
        }
        Err(e) => Err(e),
    }
}

fn is_dns_error(error: &str) -> bool {
    error.contains("could not be resolved") ||
    error.contains("unable to resolve") ||
    error.contains("DNS") ||
    error.contains("name resolution failed")
}
```

### Step 2.3: Improve Error Handling (2-3 hours)

**Create new file:** `nameback-core/src/deps/error.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepsError {
    #[error("Package manager '{0}' failed: {1}")]
    PackageManagerFailed(String, String),

    #[error("Download failed: {0}")]
    DownloadFailed(#[from] reqwest::Error),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("DNS configuration failed: {0}")]
    DnsConfigFailed(String),

    #[error("All installation methods failed for {dependency}")]
    AllMethodsFailed {
        dependency: String,
        scoop_error: Option<String>,
        chocolatey_error: Option<String>,
        bundled_error: Option<String>,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0} is required but could not be installed")]
    RequiredDependencyFailed(String),
}

pub type Result<T> = std::result::Result<T, DepsError>;
```

**Update Cargo.toml:**
```toml
[dependencies]
thiserror = "1.0"
```

**Convert all Result<(), String> to Result<()>:**
```rust
// Before
pub fn run_installer() -> Result<(), String> { ... }

// After
pub fn run_installer() -> deps::Result<()> { ... }
```

### Step 2.4: Extract Bundled Platform Logic (1-2 hours)

**Create files:**
- `nameback-core/src/deps/bundled/mod.rs`
- `nameback-core/src/deps/bundled/windows.rs`
- `nameback-core/src/deps/bundled/macos.rs`
- `nameback-core/src/deps/bundled/linux.rs`

**`bundled/mod.rs`:**
```rust
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

use reqwest::blocking;
use std::path::{Path, PathBuf};

pub fn install_from_bundled(dep_name: &str, platform: &str) -> Result<(), String> {
    let installer_path = download_from_github(dep_name, platform)?;

    #[cfg(target_os = "windows")]
    return windows::install(&installer_path, dep_name);

    #[cfg(target_os = "macos")]
    return macos::install(&installer_path, dep_name);

    #[cfg(target_os = "linux")]
    return linux::install(&installer_path, dep_name);
}

fn download_from_github(dep_name: &str, platform: &str) -> Result<PathBuf, String> {
    // Extract download logic here
}
```

**`bundled/windows.rs`:**
```rust
pub fn install(installer_path: &Path, dep_name: &str) -> Result<(), String> {
    let extract_dir = extract_archive(installer_path)?;

    match dep_name {
        "tesseract" => install_tesseract_setup(&extract_dir),
        _ => install_portable(&extract_dir, dep_name),
    }
}

fn extract_archive(path: &Path) -> Result<PathBuf, String> {
    // PowerShell extraction logic
}

fn install_tesseract_setup(extract_dir: &Path) -> Result<(), String> {
    // Run .exe installer
}

fn install_portable(extract_dir: &Path, dep_name: &str) -> Result<(), String> {
    // Copy to %LOCALAPPDATA%\Nameback\<dep>
}
```

---

## PHASE 3: Structural Refactoring (6 hours)

### Step 3.1: Create Module Structure (1 hour)

**Create directory structure:**
```bash
mkdir -p nameback-core/src/deps/{windows,macos,linux,bundled}
```

**Create stub files:**
```rust
// nameback-core/src/deps/mod.rs
pub mod constants;
pub mod progress;
pub mod dns_fallback;
pub mod bundled;
pub mod error;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;

// Re-export public API
pub use error::{DepsError, Result};
pub use progress::{ProgressReporter, ProgressCallback};

// Public API
pub fn is_command_available(command: &str) -> bool { ... }
pub fn check_dependencies() -> Vec<(Dependency, bool)> { ... }
pub fn run_installer() -> Result<()> { ... }
pub fn run_installer_with_progress(progress: Option<ProgressCallback>) -> Result<()> { ... }
```

### Step 3.2: Split Windows Code (2 hours)

**Files to create:**
- `deps/windows/mod.rs` - Entry point, calls installer functions
- `deps/windows/scoop.rs` - Scoop installation logic (~200 lines)
- `deps/windows/chocolatey.rs` - Chocolatey installation logic (~200 lines)
- `deps/windows/dns_fallback.rs` - Windows DNS switching (~100 lines)
- `deps/windows/installer.rs` - Main fallback coordinator (~150 lines)

**Extract pattern:**
1. Copy relevant sections from deps.rs
2. Remove from deps.rs
3. Import in windows/mod.rs
4. Test compilation

### Step 3.3: Split macOS Code (1.5 hours)

**Files to create:**
- `deps/macos/mod.rs` - Entry point
- `deps/macos/homebrew.rs` - Homebrew installation (~150 lines)
- `deps/macos/macports.rs` - MacPorts fallback (~100 lines)
- `deps/macos/dns_fallback.rs` - macOS DNS switching (~100 lines)

### Step 3.4: Split Linux Code (1.5 hours)

**Files to create:**
- `deps/linux/mod.rs` - Entry point
- `deps/linux/apt.rs` - apt/dpkg (~100 lines)
- `deps/linux/dnf.rs` - dnf/rpm (~100 lines)
- `deps/linux/pacman.rs` - Arch Linux (~50 lines)
- `deps/linux/dns_fallback.rs` - Linux DNS switching (~80 lines)

### Step 3.5: Final Integration (1 hour)

**Update `nameback-core/src/lib.rs`:**
```rust
pub mod deps;
pub use deps::{check_dependencies, install_dependencies, is_command_available};
```

**Ensure backward compatibility:**
```rust
// Old API still works
pub fn install_dependencies() -> Result<()> {
    deps::run_installer()
}
```

---

## Verification Checklist

After each phase, run:

```bash
# Check compilation
cargo check --workspace

# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy -p nameback-core

# Verify no dead code
cargo clippy -p nameback-core 2>&1 | grep "warning:" | wc -l
# Should be 0

# Verify line count reduction
wc -l nameback-core/src/deps.rs
# Should be ~600 (down from 1,861)

# Count new files
find nameback-core/src/deps -type f -name "*.rs" | wc -l
# Should be ~20 files
```

---

## Testing Strategy

**Unit Tests:**
- Add tests for each extracted function
- Mock external commands where possible
- Test error paths

**Integration Tests:**
- Test full fallback chain (mock downloads)
- Test DNS fallback (mock DNS commands)
- Test bundled installer (mock extraction)

**Manual Tests:**
- Windows: Test in VM with blocked Scoop
- macOS: Test with blocked Homebrew
- Linux: Test with blocked apt

---

## Rollback Plan

If refactoring causes issues:

```bash
# Revert to last working state
git diff nameback-core/src/deps.rs > /tmp/refactor.patch
git checkout nameback-core/src/deps.rs

# Apply only Phase 1 changes
git apply /tmp/phase1.patch
```

**Keep backups:**
```bash
cp nameback-core/src/deps.rs nameback-core/src/deps.rs.backup
```

---

## Progress Tracking

**Phase 1: Quick Wins**
- [ ] 1.1 Remove dead code
- [ ] 1.2 Centralize constants
- [ ] 1.3 Centralize progress reporting
- [ ] Verify all tests pass

**Phase 2: Core Refactoring**
- [ ] 2.1 Extract duplicate installation logic
- [ ] 2.2 Create DNS fallback trait
- [ ] 2.3 Improve error handling
- [ ] 2.4 Extract bundled platform logic
- [ ] Verify all tests pass

**Phase 3: Structural**
- [ ] 3.1 Create module structure
- [ ] 3.2 Split Windows code
- [ ] 3.3 Split macOS code
- [ ] 3.4 Split Linux code
- [ ] 3.5 Final integration
- [ ] Verify all tests pass
- [ ] Run manual tests on all platforms

---

## Expected Outcome

**Before:**
- deps.rs: 1,861 lines
- All code in single file
- Massive duplication
- Hard to navigate

**After:**
- deps/mod.rs: ~150 lines
- deps/windows/: ~650 lines (split across 5 files)
- deps/macos/: ~450 lines (split across 4 files)
- deps/linux/: ~380 lines (split across 5 files)
- deps/bundled/: ~200 lines (split across 4 files)
- deps/constants.rs: ~50 lines
- deps/progress.rs: ~50 lines
- deps/dns_fallback.rs: ~100 lines
- deps/error.rs: ~50 lines

**Total:** ~2,030 lines (organized into 20+ files)
**Duplication removed:** ~900 lines
**Net increase:** ~170 lines (from better structure)

---

## Notes for Resumption

**Current state when paused:**
- Clippy auto-fixes applied ✅
- Starting Phase 1.1 (remove dead code)
- No breaking changes made yet
- All tests still passing

**First steps to resume:**
1. Review this plan
2. Run `cargo test --workspace` to verify current state
3. Continue with Phase 1.1 (remove dead code)
4. Work through checklist systematically

**Estimated remaining time:** 17-20 hours

---

**Last Updated:** 2025-11-02
**Author:** Claude Code
**Repository:** https://github.com/h4x0r/nameback
