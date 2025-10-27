# Release Notes Template

## Linux Installation Guide

### Recommended: Debian Package (.deb) - Easiest Option ⭐

**Best for:** Ubuntu, Debian, Kali Linux, Linux Mint users who want everything pre-configured

```bash
wget https://github.com/h4x0r/nameback/releases/download/v{VERSION}/nameback_{VERSION}-1_amd64.deb
sudo dpkg -i nameback_{VERSION}-1_amd64.deb
sudo apt-get install -f  # Auto-installs all dependencies
```

**What you get:**
- ✅ Both CLI (`nameback`) and GUI (`nameback-gui`) installed
- ✅ All dependencies auto-installed (exiftool, tesseract, ffmpeg, imagemagick)
- ✅ Desktop entry for GUI (launches from Applications menu)
- ✅ System integration (man pages, proper paths)
- ✅ Easy uninstall: `sudo apt remove nameback`

**Why choose this:** One-command installation with zero manual setup. Everything just works.

---

### Alternative: Cargo Install - For CLI Power Users

**Best for:** Rust developers, CLI-only users, or non-Debian distributions

#### CLI Only (Recommended for automation/scripting)
```bash
cargo install nameback
nameback --install-deps  # Interactive dependency installation
```

**What you get:**
- ✅ Latest CLI tool from crates.io
- ✅ Faster updates (published immediately)
- ⚠️  GUI not included
- ⚠️  Manual dependency management

#### GUI Only (For visual workflow)
```bash
cargo install nameback --bin nameback-gui
nameback-gui
```

**What you get:**
- ✅ Latest GUI application
- ⚠️  CLI not included
- ⚠️  No desktop integration (manual launcher setup)
- ⚠️  Manual dependency management

#### Both CLI + GUI
```bash
cargo install nameback
cargo install nameback --bin nameback-gui
nameback --install-deps
```

**Why choose Cargo:**
- You're on Arch, Fedora, or other non-Debian distro
- You want bleeding-edge updates
- You prefer Rust tooling
- You only need CLI for automation

---

### Quick Comparison

| Feature | .deb Package | Cargo Install CLI | Cargo Install GUI |
|---------|--------------|-------------------|-------------------|
| **CLI tool** | ✅ | ✅ | ❌ |
| **GUI app** | ✅ | ❌ | ✅ |
| **Auto-install deps** | ✅ | ⚠️ Interactive | ⚠️ Manual |
| **Desktop integration** | ✅ | ❌ | ❌ |
| **Easy uninstall** | ✅ | ⚠️ Manual | ⚠️ Manual |
| **Debian/Ubuntu** | ⭐ Recommended | Alternative | Alternative |
| **Other distros** | ❌ | ⭐ Recommended | ⭐ Recommended |

---

## Platform-Specific Install Summary

### macOS
```bash
brew tap h4x0r/nameback
brew install nameback        # CLI only
brew install --cask nameback # GUI only (installs to /Applications)
```
**Recommended:** Use `brew install --cask nameback` for GUI with full macOS integration

### Windows
Download `nameback-x86_64-pc-windows-msvc.msi` from releases
- ✅ Both CLI + GUI included
- ✅ All dependencies auto-installed
- ✅ Start Menu integration

### Linux
**Debian/Ubuntu/Kali:** Use `.deb` package (recommended)
**Other distros:** Use `cargo install` (CLI or GUI)

---

## Security & Verification

All release artifacts include:
- **SHA256 checksums** - Verify file integrity
- **SLSA attestations** - Verify build provenance

### Verify Downloads

**Linux (.deb):**
```bash
sha256sum -c checksums.txt --ignore-missing
gh attestation verify nameback_*_amd64.deb --owner h4x0r
```

**macOS (.dmg):**
```bash
shasum -a 256 -c checksums.txt --ignore-missing
gh attestation verify nameback-*.dmg --owner h4x0r
```

**Windows (.msi):**
```powershell
# Download checksums.txt first, then:
Get-FileHash nameback-x86_64-pc-windows-msvc.msi -Algorithm SHA256
# Compare with value in checksums.txt
```

---

## Example Release Notes (v0.6.18)

### New Features
- 🎨 Professional DMG background with installation instructions
- 🏢 Security Ronin branding in macOS installer
- 📦 Enhanced Linux .deb package with desktop integration

### Improvements
- ⚡ Improved dependency detection in GUI
- 🔧 Better error messages for missing dependencies
- 📝 Clearer installation documentation

### Bug Fixes
- 🐛 Fixed cache file appearing in rename lists
- 🔤 Hebrew/Unicode text now displays correctly in GUI
- 🍎 Fixed DMG creation race condition on Apple Silicon

### Installation (Choose Your Platform)

**macOS (Recommended: Homebrew Cask)**
```bash
brew install --cask nameback  # GUI with full macOS integration
```

**Windows (MSI Installer)**
- Download: `nameback-x86_64-pc-windows-msvc.msi`
- Both CLI + GUI included
- All dependencies auto-installed

**Linux - Debian/Ubuntu/Kali (Recommended: .deb Package)**
```bash
wget https://github.com/h4x0r/nameback/releases/download/v0.6.18/nameback_0.6.18-1_amd64.deb
sudo dpkg -i nameback_0.6.18-1_amd64.deb
sudo apt-get install -f
```
✅ Both CLI + GUI
✅ All dependencies auto-installed
✅ Desktop integration included

**Linux - Other Distributions (Cargo)**
```bash
# CLI only
cargo install nameback

# GUI only
cargo install nameback --bin nameback-gui

# Then install dependencies
nameback --install-deps
```

**From Source (All Platforms)**
```bash
git clone https://github.com/h4x0r/nameback
cd nameback
cargo build --release --workspace
```

### Full Changelog
See [CHANGELOG.md](CHANGELOG.md) for complete version history

---

## Notes for Release Creators

When creating a new release, customize the template above:

1. Replace `{VERSION}` with actual version (e.g., `0.6.18`)
2. Update the "New Features" / "Improvements" / "Bug Fixes" sections
3. Ensure .deb download links point to the correct release tag
4. Verify all checksums and attestations are generated
5. Test installation on at least one Debian/Ubuntu system before publishing

The key messaging:
- **Debian users:** Use .deb (easiest, recommended)
- **Other Linux:** Use cargo install (more flexibility)
- **macOS:** Use Homebrew Cask for GUI (best integration)
- **Windows:** Use MSI installer (everything included)
