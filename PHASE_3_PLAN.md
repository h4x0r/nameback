# Phase 3 Refactoring - Detailed Plan

## Current Status (After Phase 3.2b)

**Completed:**
- ✅ Phase 3.1: Module structure created
- ✅ Phase 3.2a: Chocolatey helpers moved (98 lines)
- ✅ Phase 3.2b: Bundled installer moved (117 lines)
- ⏳ Phase 3.2c: Scoop installation (PENDING - see below)

**Current State:**
- deps.rs: 1,546 lines (down from 1,861, 17% reduction)
- Windows section: Lines 251-956 (~705 lines remaining)

## Challenge: Windows Section is Too Large

The Windows Scoop installation section (705 lines) is too large to move as a single function. It contains:
- DNS fallback closures (94 lines)
- Scoop installation logic (194 lines)
- Individual dependency installations: exiftool, tesseract, ffmpeg, imagemagick (417 lines)

### Recommended Approach for Phase 3.2c

Instead of moving the entire Windows section to one file, **split it into focused functions:**

#### Option A: Keep Current Structure (Simplest)
Leave the Windows installation logic in deps.rs for now. It's already well-organized with closures and works correctly.

**Pros:**
- No risk of breaking functionality
- Tests continue passing
- Can revisit after macOS/Linux refactoring

**Cons:**
- deps.rs still large (~1,546 lines)
- Windows logic not as modular

#### Option B: Incremental Extraction (Recommended)
Extract smaller pieces progressively:

1. **Move DNS helpers to `deps/windows/dns_fallback.rs`** (~94 lines)
   ```rust
   pub fn try_with_public_dns() -> Result<(), String>
   pub fn restore_dns()
   ```

2. **Create `deps/windows/scoop_installer.rs`** for Scoop setup (~194 lines)
   ```rust
   pub fn ensure_scoop_installed(report_progress: impl Fn(&str, u8)) -> Result<String, String>
   // Returns: scoop.cmd path
   ```

3. **Update `deps/windows/scoop.rs`** to use dependencies (~417 lines)
   ```rust
   pub fn install_via_scoop(progress: &Option<ProgressCallback>) -> Result<(), String>
   ```

This would reduce deps.rs by ~705 lines to ~841 lines.

#### Option C: Full Rewrite (Most Complex)
Completely redesign the Windows installation flow with proper separation of concerns. This would take significant time and risk introducing bugs.

## Recommendation for Next Session

**Choose Option B**: Incremental extraction

**Steps:**
1. Extract DNS fallback to `deps/windows/dns_fallback.rs`
2. Extract Scoop installer to `deps/windows/scoop_installer.rs`
3. Move remaining logic to `deps/windows/scoop.rs`
4. Update deps.rs to call `windows::install_via_scoop(progress)`
5. Run tests, commit

**Estimated effort:** 2-3 hours
**Expected reduction:** deps.rs 1,546 → ~841 lines (705 lines moved)

## Alternative: Skip Windows, Do macOS/Linux First

If Windows refactoring is too complex, we could:
1. Move to Phase 3.3 (macOS) - simpler, ~350 lines
2. Move to Phase 3.4 (Linux) - simpler, ~330 lines
3. Come back to Windows later

This would demonstrate the pattern and make progress on other platforms.

## Final Target

After all phases complete:
- deps.rs: ~300-400 lines (just orchestration)
- Platform modules: ~100-150 lines each
- Total codebase: same functionality, better organization

## Token Budget Note

Moving the full 705-line Windows section requires careful handling. The code is already at 147K/200K tokens (73.5%). Recommend:
- Commit current progress
- Start fresh session for Phase 3.2c
- Use incremental approach (Option B)
