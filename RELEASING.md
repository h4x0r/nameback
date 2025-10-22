# Release Process

This project uses [cargo-release](https://github.com/crate-ci/cargo-release) for version management and publishing.

## Quick Start

```bash
# Install cargo-release (one time)
cargo install cargo-release

# Preview a patch release (0.5.0 → 0.5.1)
cargo release patch --dry-run

# Execute a patch release
cargo release patch --execute

# Preview a minor release (0.5.0 → 0.6.0)
cargo release minor --dry-run

# Execute a minor release
cargo release minor --execute

# Preview a major release (0.5.0 → 1.0.0)
cargo release major --dry-run

# Execute a major release
cargo release major --execute
```

## What cargo-release Does

When you run `cargo release <version> --execute`, it will:

1. ✅ **Update all version numbers automatically**
   - `Cargo.toml` workspace.package.version
   - `Cargo.toml` workspace.dependencies (nameback-core version)
   - All three package Cargo.toml files (via workspace inheritance)

2. ✅ **Run tests** - Ensures everything passes before release

3. ✅ **Create git commit** - `chore: release v0.6.0`

4. ✅ **Create git tag** - `v0.6.0`

5. ✅ **Publish to crates.io** in dependency order:
   - First: `nameback-core`
   - Second: `nameback` (CLI)
   - Third: `nameback-gui`

6. ✅ **Push to GitHub** - Triggers binary release workflow

7. ✅ **Binary release workflow runs automatically**:
   - Builds Windows MSI installer
   - Builds macOS DMG files (Intel + Apple Silicon)
   - Builds Linux .deb package
   - Generates SLSA attestations
   - Creates GitHub Release with all artifacts

## No More Manual Version Updates! 🎉

**Before cargo-release:**
```toml
# ❌ Had to manually update in 2 places:
[workspace.package]
version = "0.5.0"  # Manual update

[workspace.dependencies]
nameback-core = { version = "0.5.0", path = "..." }  # Manual update
```

**After cargo-release:**
```bash
# ✅ Just run ONE command:
cargo release patch --execute

# cargo-release automatically updates:
# - workspace.package.version
# - workspace.dependencies.nameback-core.version
# - All package versions (via workspace = true)
```

## Version Bumping Rules

### Patch Release (0.5.0 → 0.5.1)
- Bug fixes
- Documentation updates
- Performance improvements (no API changes)
- Security patches

```bash
cargo release patch --execute
```

### Minor Release (0.5.0 → 0.6.0)
- New features (backward compatible)
- New public APIs
- Deprecations (but not removals)

```bash
cargo release minor --execute
```

### Major Release (0.5.0 → 1.0.0)
- Breaking API changes
- Removal of deprecated features
- Major architectural changes

```bash
cargo release major --execute
```

## Dry Run (Always Recommended)

**Always preview first:**
```bash
cargo release minor --dry-run
```

This shows you:
- What version changes will be made
- Which files will be modified
- What git operations will happen
- Publishing order

## Release Workflow Diagram

```
You run cargo-release
         ↓
Updates all Cargo.toml versions
         ↓
Runs cargo test
         ↓
Creates git commit + tag (v0.6.0)
         ↓
Publishes nameback-core to crates.io
         ↓
Publishes nameback (CLI) to crates.io
         ↓
Publishes nameback-gui to crates.io
         ↓
Pushes tag to GitHub
         ↓
GitHub Actions release workflow triggers
         ↓
Builds Windows MSI
         ↓
Builds macOS DMG (Intel + ARM)
         ↓
Builds Linux .deb
         ↓
Generates SLSA attestations
         ↓
Creates GitHub Release
         ↓
✅ DONE!
```

## Configuration

The `release.toml` file controls cargo-release behavior:

```toml
[workspace]
allow-branch = ["main"]           # Only release from main
consolidate-commits = true        # Single commit for all packages
tag-name = "v{{version}}"         # Tag format
dependent-version = "upgrade"     # Update workspace deps automatically
```

## Prerequisites

### One-Time Setup

1. **Install cargo-release:**
   ```bash
   cargo install cargo-release
   ```

2. **Configure crates.io token:**
   ```bash
   cargo login
   # Enter your crates.io API token
   ```

3. **Ensure clean git state:**
   ```bash
   git status  # Should show "nothing to commit, working tree clean"
   ```

### GitHub Secrets (Already Configured)

- `CARGO_REGISTRY_TOKEN` - For automated crates.io publishing in CI

## Common Workflows

### Standard Release

```bash
# 1. Ensure you're on main with latest changes
git checkout main
git pull origin main

# 2. Preview the release
cargo release minor --dry-run

# 3. Execute if everything looks good
cargo release minor --execute

# 4. Monitor GitHub Actions
gh run list --workflow=release.yml --limit 1
```

### Emergency Patch Release

```bash
# Quick patch release for critical bug fix
git checkout main
git pull origin main
cargo release patch --execute
```

### Pre-release Testing

```bash
# Test locally before releasing
cargo test --workspace
cargo build --release --workspace
cargo doc --workspace --no-deps

# Then release
cargo release patch --execute
```

## Troubleshooting

### "uncommitted changes" error

```bash
# Commit or stash your changes first
git status
git add .
git commit -m "feat: your changes"
```

### "not on allowed branch" error

```bash
# Switch to main branch
git checkout main
git pull origin main
```

### Publishing fails

Check:
- `cargo login` token is valid
- You have publish permissions for nameback-* crates
- No version conflicts on crates.io

### Tests fail during release

```bash
# Fix tests first
cargo test --workspace

# Then retry release
cargo release patch --execute
```

### GitHub Actions release fails

Check:
- Workflow logs: `gh run view --log-failed`
- CARGO_REGISTRY_TOKEN secret is set
- Build dependencies installed correctly

## Manual Override (Emergency Only)

If cargo-release fails and you need to release manually:

```bash
# 1. Update versions manually in Cargo.toml files
# 2. Commit changes
git add Cargo.toml */Cargo.toml Cargo.lock
git commit -m "chore: release v0.5.1"

# 3. Create and push tag
git tag -a v0.5.1 -m "Release v0.5.1"
git push origin main
git push origin v0.5.1

# 4. Publish manually (in dependency order)
cd nameback-core && cargo publish && cd ..
cd nameback-cli && cargo publish && cd ..
cd nameback-gui && cargo publish && cd ..
```

## Verifying Releases

### Check crates.io

```bash
# View published versions
cargo search nameback
```

Or visit:
- https://crates.io/crates/nameback
- https://crates.io/crates/nameback-core
- https://crates.io/crates/nameback-gui

### Check GitHub Release

```bash
gh release view v0.5.1
```

### Verify SLSA Attestations

```bash
# Download artifact and verify
gh attestation verify nameback-x86_64-pc-windows-msvc.msi --owner h4x0r
```

## Learn More

- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [Semantic Versioning](https://semver.org/)
- [crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
