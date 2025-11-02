# Nameback Complete Guide

This guide covers installation options, advanced features, and troubleshooting for nameback.

## Table of Contents

- [Installation Options](#installation-options)
- [Dependency Management](#dependency-management)
- [Advanced Features](#advanced-features)
- [Metadata Extraction Details](#metadata-extraction-details)
- [Safety & Security](#safety--security)
- [Troubleshooting](#troubleshooting)

---

## Installation Options

### macOS

#### Homebrew (Recommended)
```bash
brew tap h4x0r/nameback
brew install nameback
```
Automatically installs all dependencies: exiftool, tesseract, ffmpeg, imagemagick

#### GUI App (DMG Installer)

**Intel Mac:**
1. Download `nameback-x86_64-apple-darwin.dmg` from [releases](https://github.com/h4x0r/nameback/releases/latest)
2. Open the DMG file
3. Drag `nameback.app` to your Applications folder
4. Launch from Applications

**Apple Silicon:**
1. Download `nameback-aarch64-apple-darwin.dmg` from [releases](https://github.com/h4x0r/nameback/releases/latest)
2. Open the DMG file
3. Drag `nameback.app` to your Applications folder
4. Launch from Applications

**Note:** For CLI usage, use Homebrew instead (see above).

### Windows

#### MSI Installer (Recommended)

Download and install `nameback-x86_64-pc-windows-msvc.msi` from [releases](https://github.com/h4x0r/nameback/releases/latest)

**What you get:**
- **CLI**: Type `nameback` in any terminal (automatically added to PATH)
- **GUI**: Launch from Start Menu → nameback
- **Auto-install dependencies**: exiftool, tesseract, ffmpeg, imagemagick

The MSI installer automatically installs all required dependencies during setup. No manual configuration needed!

**Silent Installation:**
```powershell
msiexec /i nameback-x86_64-pc-windows-msvc.msi /quiet
```

The installer uses custom actions for dependency checking and installation. Console windows are automatically hidden during silent installation for a clean user experience.

### Linux

#### CLI Tool via Cargo (Recommended)
```bash
cargo install nameback
nameback --install-deps  # Interactive dependency installation
```

This installs the command-line tool only. Dependencies are installed automatically when you run `nameback --install-deps` or on first use (smart detection prompts you).

#### GUI Application via Cargo
```bash
cargo install nameback --bin nameback-gui
nameback-gui
```

This installs the GUI application. You can launch it from the terminal or create a desktop shortcut.

#### Debian/Ubuntu/Kali (.deb Package) - Includes CLI + GUI
```bash
# Download the latest .deb package
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback_0.4.1_amd64.deb

# Install with dependencies
sudo dpkg -i nameback_0.4.1_amd64.deb
sudo apt-get install -f

# Now you have both:
nameback          # CLI tool
nameback-gui      # GUI application
```

The `.deb` package automatically installs all dependencies and creates desktop menu entries for the GUI.

**Dependencies included:**
- `libimage-exiftool-perl` - Metadata extraction (required)
- `tesseract-ocr` - OCR for images and videos
- `tesseract-ocr-chi-tra` - Traditional Chinese language support
- `tesseract-ocr-chi-sim` - Simplified Chinese language support
- `ffmpeg` - Video frame extraction
- `imagemagick` - HEIC/HEIF image support

#### Manual Dependency Installation (Advanced)

If you prefer to manage dependencies manually:

**Debian/Ubuntu/Kali:**
```bash
sudo apt-get install libimage-exiftool-perl tesseract-ocr tesseract-ocr-chi-tra tesseract-ocr-chi-sim ffmpeg imagemagick
```

**Fedora/RHEL:**
```bash
sudo dnf install perl-Image-ExifTool tesseract tesseract-langpack-chi_tra tesseract-langpack-chi_sim ffmpeg imagemagick
```

**Arch Linux:**
```bash
sudo pacman -S perl-image-exiftool tesseract tesseract-data-chi_tra tesseract-data-chi_sim ffmpeg imagemagick
```

### Build from Source

This project uses a Cargo workspace with 3 crates:
- **`nameback-core`** - Shared library with core rename logic
- **`nameback-cli`** - Command-line tool
- **`nameback-gui`** - GUI application (egui-based)

#### Build All

```bash
# Clone the repository
git clone https://github.com/h4x0r/nameback.git
cd nameback

# Build everything
cargo build --release

# Binaries:
# target/release/nameback (CLI)
# target/release/nameback-gui (GUI)
```

#### Build Individual Tools

```bash
# CLI only
cargo build --release -p nameback

# GUI only
cargo build --release -p nameback-gui

# Core library only
cargo build --release -p nameback-core
```

#### Running Tests

```bash
cargo test --workspace
```

---

## Dependency Management

### Required Dependencies

- **exiftool** - Core metadata extraction (always required)

### Optional Dependencies

These are automatically installed with `nameback --install-deps`:

- **tesseract-ocr** - Enables OCR for images/videos without metadata
- **ffmpeg** - Enables video frame extraction for OCR
- **ImageMagick** - Enables HEIC/HEIF support on Windows/Linux (macOS has native support via `sips`)

### Checking Dependencies

```bash
nameback --check-deps
```

### What Works Without Optional Dependencies

Without tesseract/ffmpeg/imagemagick, nameback still works for:
- Photos with EXIF metadata (JPEG, PNG)
- PDFs with title metadata or embedded text
- Office documents (DOCX, XLSX, PPTX)
- Audio files (MP3, FLAC, etc.)

You'll miss out on:
- OCR for screenshots without metadata
- Video frame OCR
- HEIC/HEIF support (Windows/Linux only)
- Scanned PDF OCR

---

## Advanced Features

### GPS Location & Timestamp Enrichment

**By default**, nameback automatically enriches filenames with GPS location and timestamps when available:

#### GPS Location (Enabled by Default)
Photos and videos with GPS metadata get location information added to their filenames:

**Default behavior (geocoding enabled):**
```
IMG_1234.jpg with GPS 37.7749N, 122.4194W
→ Sunset_at_the_Beach_San_Francisco_CA_2024-03-15.jpg
```

**With `--no-geocode` (raw coordinates):**
```
→ Sunset_at_the_Beach_37.77N_122.42W_2024-03-15.jpg
```

**Disable location entirely with `--no-location`:**
```
→ Sunset_at_the_Beach_2024-03-15.jpg
```

#### Timestamp Enrichment (Enabled by Default)
Timestamps are automatically added when available in EXIF metadata:

```
Screenshot.png with DateTimeOriginal: 2024-03-15 14:30:22
→ Screenshot_2024-03-15.png
```

**Disable with `--no-timestamp`:**
```
→ Screenshot.png
```

#### Geocoding Details
- Uses **OpenStreetMap Nominatim API** (free, no API key required)
- Automatically rate-limited to comply with usage policy (1 req/sec)
- Results cached for 1 hour to minimize API calls
- Falls back to coordinates if geocoding fails (offline, rate limit, etc.)
- US/Canada locations abbreviated: "Seattle_WA", "Toronto_ON"
- International locations: "Paris_France", "Tokyo_Japan"

### Multi-Language OCR

When tesseract-lang is installed, nameback automatically tries multiple languages and selects the best result:

- **Traditional Chinese** (`chi_tra`) - Tried first for broader compatibility
- **Simplified Chinese** (`chi_sim`) - Second priority
- **English** (`eng`) - Always available as fallback
- **160+ additional languages** available via tesseract-lang

The system automatically selects the language that produces the most characters, ensuring optimal text extraction.

**Example:**
```
screenshot.png with Chinese text → Tries chi_tra, chi_sim, eng → Selects chi_tra (120 chars)
→ Renames to: 輸入_姓名_電話號碼.png
```

### HEIC/HEIF Support

Apple's High Efficiency Image Format is fully supported:

- **macOS**: Uses `sips` (built-in) for fast conversion
- **Windows/Linux**: Uses ImageMagick for conversion
- Extracts EXIF metadata before OCR
- Maintains original `.heic` extension after renaming

**Process:**
1. Extract EXIF metadata (Title, Description, DateTimeOriginal)
2. If no metadata, convert HEIC → PNG using sips/ImageMagick
3. Run OCR on converted image
4. Rename original HEIC file

### Video Frame OCR

For videos without useful metadata:

1. Extracts frame at **1 second** using ffmpeg
2. Runs multi-language OCR on the frame
3. Uses first 80 characters as filename

**Supported formats:** MP4, MOV, AVI, MKV, WebM, FLV, WMV, M4V

**Use cases:**
- Screen recordings with text
- Tutorial videos with title screens
- Presentation recordings

**Example:**
```
VID_20241015.mp4 (no metadata) → Extract frame at 1s → OCR detects "Product Demo"
→ Renames to: Product_Demo.mp4
```

---

## Metadata Extraction Details

Nameback uses intelligent heuristics to extract meaningful names from various file types.

### How It Works

Each file type has a **priority-based extraction system**:

1. **Images** (JPEG, PNG, HEIC, etc.): Title → Description → DateTimeOriginal → OCR
2. **PDFs**: Title → Subject → Text extraction → OCR (scanned)
3. **Videos**: Title → CreationDate → Multi-frame OCR
4. **Office Docs**: Title → Subject → Author (filtered)
5. **Audio**: Title → Artist → Album

### Quality Filtering

Names are validated with **quality scoring** to ensure meaningful results:
- Rejects scanner/printer names ("Canon iPR C165")
- Filters generic placeholders ("Untitled", "Document1")
- Validates minimum length and character diversity
- Falls back to original filename if quality too low

### Multi-Language OCR

When metadata is missing, OCR tries multiple languages:
- Traditional Chinese (`chi_tra`)
- Simplified Chinese (`chi_sim`)
- English (`eng`)
- Selects the language with most extracted characters

### Supported File Types

**Renamed automatically** (when they have useful metadata):
- Photos: JPEG, PNG, HEIC/HEIF, GIF, TIFF
- Documents: PDF, DOCX, XLSX, PPTX, ODT, ODS, ODP
- Videos: MP4, MOV, AVI, MKV, WebM
- Audio: MP3, FLAC, WAV, OGG, M4A
- Web: HTML, MHTML (email)
- Archives: ZIP, TAR (based on contents)

**Skipped** (no useful metadata):
- Plain text: TXT, CSV, MD
- System files: DLL, EXE, temp files
- Recovered files without metadata

For complete technical details and examples, see [Intelligent Naming Heuristics](naming-heuristics.md).

---

## Safety & Security

### Verifying Downloaded Artifacts

All nameback release artifacts are signed with **SLSA build provenance attestations** for supply chain security. This ensures that your downloaded files are authentic and haven't been tampered with.

#### Why Verify?

- **Authenticity** - Proves the artifact was built by the official h4x0r/nameback repository
- **Integrity** - Confirms the file hasn't been modified since it was built
- **Transparency** - Shows the exact commit SHA that built the artifact
- **Trust** - Validates the build came from the official GitHub Actions workflow

#### Prerequisites

Install the GitHub CLI:
- **macOS**: `brew install gh`
- **Windows**: Download from https://cli.github.com/
- **Linux**: See https://github.com/cli/cli/blob/trunk/docs/install_linux.md

#### Verifying Artifacts

**Windows MSI Installer:**
```bash
gh attestation verify nameback-x86_64-pc-windows-msvc.msi --owner h4x0r
```

**macOS DMG Installers:**
```bash
# Intel Mac
gh attestation verify nameback-x86_64-apple-darwin.dmg --owner h4x0r

# Apple Silicon Mac
gh attestation verify nameback-aarch64-apple-darwin.dmg --owner h4x0r
```

**Linux Debian Package:**
```bash
gh attestation verify nameback_0.5.0-1_amd64.deb --owner h4x0r
```

**Windows ZIP Archive:**
```bash
gh attestation verify nameback-x86_64-pc-windows-msvc.zip --owner h4x0r
```

#### What the Verification Shows

When verification succeeds, you'll see:
```
✓ Verification succeeded!

sha256:abc123... was attested by:
REPO        PREDICATE_TYPE                  WORKFLOW
h4x0r/nameback  https://slsa.dev/provenance/v1  .github/workflows/release.yml@refs/tags/v0.5.0
```

This confirms:
- ✅ The artifact was built by the official h4x0r/nameback repository
- ✅ It was built from the official release.yml workflow
- ✅ It matches the exact commit SHA shown
- ✅ It hasn't been tampered with since build

#### Additional Verification with Checksums

For maximum security, combine attestation verification with checksum verification:

```bash
# Download checksums.txt from the release
wget https://github.com/h4x0r/nameback/releases/latest/download/checksums.txt

# Verify checksum matches
sha256sum -c checksums.txt
```

**Two-layer security:**
- **Attestations** verify authenticity (came from official repo/workflow)
- **Checksums** verify integrity (file not corrupted/modified)

Using **both** provides the strongest security guarantee.

### Operational Safety

- **Preview mode** - Always test with `--dry-run` first
- **No overwrites** - Skips files if destination already exists
- **Duplicate handling** - Adds `_1`, `_2` suffixes automatically
- **Filename sanitization** - Removes special characters safely

### Security Constraints

- **Same-directory only** - Files can only be renamed within their parent directory (no path traversal)
- **Permission-based** - Respects standard Unix/filesystem permissions
- **No privilege escalation** - Cannot access system directories without appropriate permissions
- **Root execution blocked** - Refuses to run as root to prevent accidental system directory modification

### Best Practices

- Always run `--dry-run` first to preview changes
- Use on user data directories where you have write permissions
- Back up important files before bulk renaming operations
- Test on a small subset before processing thousands of files

### How It Works

1. **Scans** - Recursively walks through directories
2. **Detects** - Identifies file types (image, document, video, audio)
3. **Extracts** - Pulls metadata using exiftool
4. **Enhances** - Falls back to OCR for files without metadata (when tesseract is available)
5. **Generates** - Creates clean, descriptive filenames
6. **Renames** - Updates files (or shows preview in dry-run mode)

---

## Troubleshooting

### No files renamed?

**Check verbose output:**
```bash
nameback ~/Pictures --verbose --dry-run
```

**Common reasons:**
- Files lack useful metadata (this is normal for many file types)
- Metadata contains only scanner/printer names (filtered out)
- Files are in formats that don't support metadata (TXT, CSV, etc.)

### Permission denied?

**Solution:**
- Ensure you have write access to the directory
- Don't run as root (nameback blocks this for safety)
- Check file permissions: `ls -la /path/to/files`

### OCR not working?

**Check dependencies:**
```bash
nameback --check-deps
```

**Install missing OCR tools:**
```bash
# macOS
brew install tesseract tesseract-lang

# Windows
nameback --install-deps

# Linux (Debian/Ubuntu)
sudo apt-get install tesseract-ocr tesseract-ocr-chi-tra tesseract-ocr-chi-sim
```

### HEIC files not working?

**macOS:** Should work out of the box using `sips`

**Windows/Linux:** Install ImageMagick
```bash
# Windows
nameback --install-deps

# Linux
sudo apt-get install imagemagick  # Debian/Ubuntu
sudo dnf install imagemagick      # Fedora/RHEL
```

### Want to undo changes?

**Important:** nameback doesn't keep backups

**Options:**
1. Use version control (git) in your directories
2. Always test with `--dry-run` first
3. Back up important files before bulk operations
4. Use filesystem snapshots (Time Machine, etc.)

### Large file sets taking too long?

**Tips:**
- Process subdirectories individually
- Use `--skip-hidden` to ignore system files
- Consider disabling OCR for initial pass (faster, uses only EXIF)

### Dependencies conflicting with other tools?

**Homebrew isolation (macOS):**
```bash
# nameback uses standard Homebrew packages
# No conflicts expected with other tools
```

**Windows isolation:**
```bash
# Tools installed via winget/Chocolatey go to standard paths
# Shouldn't conflict with existing installations
```

---

## Command-Line Options

### Basic Usage
```bash
nameback <directory>              # Rename files in directory
nameback <directory> --dry-run    # Preview changes only
```

### Flags

- `--dry-run` or `-n` - Preview changes without modifying files
- `--verbose` or `-v` - Show detailed progress and decisions
- `--skip-hidden` or `-s` - Skip hidden files (like `.DS_Store`)
- `--check-deps` - Check dependency installation status
- `--install-deps` - Install missing dependencies interactively

### Examples

```bash
# Preview changes in Photos folder
nameback ~/Pictures --dry-run

# Process with detailed logging
nameback ~/Documents --verbose

# Skip hidden files
nameback ~/Downloads --skip-hidden

# Check what dependencies are installed
nameback --check-deps
```

---

## License

MIT License - see [LICENSE](LICENSE) file for details

Created by Albert Hui ([@4n6h4x0r](https://github.com/h4x0r))

Built with [Claude Code](https://claude.com/claude-code)
