# Additional Guidance

This file provides instructions for Claude Code when working on the Nameback project.

## Project Overview

Nameback is a file renaming utility that intelligently renames files based on their metadata. It provides both a command-line interface (CLI) and a graphical user interface (GUI) for renaming files using metadata extraction, OCR, and content analysis.

## Architecture

The project is implemented in Rust as a Cargo workspace with three packages:

### Workspace Structure

- **nameback-core** - Shared library containing all core functionality
- **nameback-cli** - Command-line interface binary
- **nameback-gui** - Graphical user interface using egui/eframe

This workspace architecture enables code reuse between CLI and GUI while maintaining clean separation of concerns. The core library handles all file processing logic, while the CLI and GUI packages provide different user interfaces to the same functionality.

**Note:** The legacy `/src` directory at the repository root still exists from the pre-workspace architecture but is no longer used. All active development happens in the workspace packages:
- nameback-core/src/ - Core library code
- nameback-cli/src/ - CLI binary code (main.rs, cli.rs)
- nameback-gui/src/ - GUI application code

### CLI Implementation (nameback-cli/src/)

The command-line interface at /Users/4n6h4x0r/src/nameback/nameback-cli/src/:

- **main.rs** - CLI entry point and orchestration
  - Argument parsing and validation
  - Logger initialization
  - Dependency checking and installation coordination
  - Windows MSI console hiding (lines 6-30)
    - Detects MSIHANDLE environment variable for MSI context
    - Validates handle as u32 for safety
    - Uses unsafe Windows API calls (GetConsoleWindow, ShowWindow)
    - Comprehensive SAFETY documentation explaining why unsafe is safe
    - Prevents console flash during silent installation
  - Root execution prevention on Unix (security measure)
  - Directory processing coordination

- **cli.rs** - CLI argument definitions using clap derive macros

**Platform-specific dependencies:**
- Windows only: windows crate v0.58
  - Win32_System_Console feature for GetConsoleWindow
  - Win32_UI_WindowsAndMessaging feature for ShowWindow
  - Conditional compilation with #[cfg(windows)]

### Core Modules (nameback-core/src/)

All core functionality lives in the `nameback-core` library at /Users/4n6h4x0r/src/nameback/nameback-core/src/:

- **lib.rs** - Public API and module coordination
  - Exports public types: RenameConfig, FileCategory, Dependency, DependencyNeeds
  - Provides high-level functions: process_directory, check_dependencies, install_dependencies
  - Default configuration with multi-frame video analysis enabled

- **detector.rs** - File type detection
  - Uses `infer` crate for magic number detection
  - Categorizes files: Image, Document, Audio, Video, Unknown
  - Handles HEIC/HEIF and other modern formats

- **extractor.rs** - Metadata extraction
  - Executes `exiftool -json` for metadata extraction
  - Parses JSON output into structured data
  - Priority-based field extraction by file category
  - Integrates OCR results for images and videos

- **generator.rs** - Filename generation
  - Sanitizes filenames (removes special characters, control chars)
  - Replaces spaces with underscores
  - Collapses multiple underscores
  - Limits length to 200 characters
  - Ensures uniqueness by appending counters (e.g., filename_1.ext)
  - Preserves original file extensions

- **renamer.rs** - Renaming engine
  - Orchestrates the full processing pipeline
  - Checks file and directory permissions
  - Prevents file overwrites (safety check)
  - Pre-populates existing filenames to prevent duplicates
  - Handles errors gracefully with logging

- **image_ocr.rs** - Image OCR processing
  - Uses tesseract-rs for text extraction
  - Supports 160+ languages including Chinese (Traditional/Simplified)
  - Extracts text from image files

- **video_ocr.rs** - Video frame OCR
  - Extracts frames at 1s, 5s, 10s intervals (multi-frame mode)
  - Single frame extraction (fast-video mode)
  - Runs OCR on each frame and selects best result
  - Uses ffmpeg for frame extraction

- **pdf_content.rs** - PDF text extraction
  - Extracts text from PDF documents
  - Falls back to OCR for scanned PDFs
  - Prioritizes metadata title over content

- **text_content.rs** - Structured text parsing
  - Markdown frontmatter extraction (YAML/TOML)
  - CSV semantic column detection
  - Nested JSON/YAML field extraction

- **code_docstring.rs** - Source code metadata
  - Extracts docstrings from Python, Rust, JavaScript, etc.
  - Parses module-level documentation

- **dir_context.rs** - Directory structure analysis
  - Analyzes parent directory names for context
  - Helps improve naming based on file organization

- **stem_analyzer.rs** - Filename analysis
  - Extracts meaningful parts from original filenames
  - Removes camera/device patterns
  - Identifies useful name components

- **format_handlers/** - Format-specific handlers
  - **archive.rs** - ZIP/TAR file analysis
  - **email.rs** - EML file parsing
  - **web.rs** - HTML/MHTML processing

- **deps.rs** - Dependency installation
  - Platform-specific package manager detection
  - Interactive dependency installation
  - Homebrew (macOS), apt/dnf (Linux), Chocolatey (Windows)
  - Windows MSI progress reporting (msi_progress module)
    - Uses MSIHANDLE environment variable for installer context
    - Reports installation progress via MsiProcessMessage API
    - Displays action start/data messages in MSI UI

- **deps_check.rs** - Dependency verification
  - Detects which external tools are needed
  - Checks if required dependencies are installed
  - Returns structured dependency status

- **scorer.rs** - Quality scoring system for candidate names (INTEGRATED)
  - Implements NameCandidate with score and source tracking
  - Multi-criteria scoring: length, specificity, language, format
  - Filters out low-quality names (device IDs, errors, generic placeholders)
  - Used by extractor.rs to intelligently select from multiple naming sources
  - Integrated in: extractor.rs:34-121, video_ocr.rs:64-125

- **series_detector.rs** - File series detection (INTEGRATED)
  - Detects numbered sequences (name_001, name_002, etc.)
  - Supports multiple patterns: underscore, parentheses, hyphen, space
  - Maintains consistent numbering across series
  - Prevents breaking existing sequences during rename
  - Integrated in: lib.rs:109-154

- **location_timestamp.rs** - GPS and timestamp enrichment (INTEGRATED)
  - Extracts GPS coordinates from EXIF data
  - Formats timestamps in configurable styles
  - Enables location-based and time-based filename components
  - Integrated in: extractor.rs:195-200, generator.rs:32-47
  - Controlled by RenameConfig.include_location and RenameConfig.include_timestamp flags

- **key_phrases.rs** - NLP-based phrase extraction (INTEGRATED)
  - Lightweight n-gram based key phrase extraction
  - Stop word filtering
  - Position-based scoring (earlier text weighted higher)
  - Length bonus for multi-word phrases
  - Automatically extracts meaningful phrases from long text (>150 chars)
  - Integrated in: pdf_content.rs:15-22,70-77, image_ocr.rs:21-29, video_ocr.rs:42-50,105-117, text_content.rs:230-236
  - Improves naming quality for OCR results, PDF content, and text files

### Dependencies

Workspace-level dependencies defined in /Users/4n6h4x0r/src/nameback/Cargo.toml:

**Core libraries:**
- anyhow 1.0 - Error handling with context
- serde 1.0 - Serialization framework
- serde_json 1.0 - JSON parsing for exiftool output
- regex 1.10 - Pattern matching and sanitization
- log 0.4 - Logging facade
- walkdir 2.4 - Recursive directory traversal
- chrono 0.4 - Date and time handling
- infer 0.16 - File type detection via magic numbers

**File format handling:**
- pdf-extract 0.7 - PDF text extraction
- image 0.25 - Image processing

**OCR:**
- tesseract 0.14 - OCR engine bindings

**CLI (nameback-cli):**
- clap 4.5 - CLI argument parsing with derive macros
- env_logger 0.11 - Logger implementation
- windows 0.58 - Windows API bindings (Windows only)
  - Win32_System_Console - Console window management
  - Win32_UI_WindowsAndMessaging - Window manipulation

**GUI (nameback-gui):**
- eframe 0.29 - egui framework
- egui 0.29 - Immediate mode GUI
- rfd 0.15 - Native file dialogs
- tokio 1.x - Async runtime for background processing

**Platform-specific:**
- libc 0.2 - System-level operations

**Testing:**
- tempfile 3.8 - Temporary file handling for tests

### External Tool Requirements

The application depends on several external command-line tools:

**Required:**
- **exiftool** - Metadata extraction (EXIF, IPTC, XMP, etc.)

**Optional (for advanced features):**
- **tesseract** - OCR for images and video frames (160+ languages)
- **ffmpeg** - Video frame extraction for video OCR
- **imagemagick** - HEIC/HEIF image format support

**Installation:**
- macOS (Homebrew): `brew install exiftool tesseract tesseract-lang ffmpeg imagemagick`
- Linux (Debian/Ubuntu): `apt-get install libimage-exiftool-perl tesseract-ocr tesseract-ocr-chi-tra tesseract-ocr-chi-sim ffmpeg imagemagick`
- Windows (MSI installer): All dependencies auto-installed during setup
- Or use: `nameback --install-deps` for interactive installation

### Error Handling

The codebase uses Result-based error handling throughout:
- anyhow::Result for propagating errors with context
- Graceful degradation: skips files with missing metadata or unknown types
- Comprehensive logging at INFO, WARN, and ERROR levels

### Safety Features

Recent bug fixes have added important safety mechanisms:
- Pre-population of existing_names HashSet prevents duplicate filename collisions
- File overwrite protection checks if destination already exists
- Permission validation before attempting renames
- Dry-run mode for safe preview before making changes
- Root execution blocked on Unix to prevent accidental system directory modification

**Windows MSI Safety:**
- Console window hiding validated with proper MSIHANDLE checking
- Unsafe Windows API calls documented with comprehensive SAFETY comments
- Environment variable validation (u32 parsing) before API usage
- No memory allocation/deallocation in unsafe blocks

### Usage Examples

**CLI tool** (see /Users/4n6h4x0r/src/nameback/nameback-cli/src/main.rs):
```bash
# Basic usage
nameback /path/to/directory

# Preview mode (no changes)
nameback --dry-run /path/to/directory

# Skip hidden files with verbose output
nameback --skip-hidden --verbose /path/to/directory

# Include GPS location in photo/video names
nameback --include-location ~/Pictures

# Use faster single-frame video analysis
nameback --fast-video /path/to/videos

# Check dependency status
nameback --check-deps

# Install missing dependencies
nameback --install-deps
```

**GUI application** (see /Users/4n6h4x0r/src/nameback/nameback-gui/src/main.rs):
- Launch via: `nameback-gui` command or from Start Menu/Applications
- Visual dual-pane interface with checkbox selection
- Real-time preview before renaming
- Color-coded status indicators

### Build and Run

The project uses a Cargo workspace structure:

```bash
# Build entire workspace (all packages)
cargo build --release --workspace

# Build specific packages
cargo build --release -p nameback-cli
cargo build --release -p nameback-gui
cargo build --release -p nameback-core

# Run CLI directly
cargo run -p nameback-cli -- /path/to/directory

# Run GUI
cargo run -p nameback-gui

# Run tests
cargo test --workspace
```

**Binary locations after build:**
- CLI: /Users/4n6h4x0r/src/nameback/target/release/nameback
- GUI: /Users/4n6h4x0r/src/nameback/target/release/nameback-gui

### Release Management

This project uses cargo-release for automated version management.

**IMPORTANT:** See [RELEASING.md](/Users/4n6h4x0r/src/nameback/RELEASING.md) for the complete, authoritative release process documentation. The guide covers cargo-release usage, GitHub Actions workflows, platform-specific builds, and troubleshooting.

**Quick release commands:**
```bash
# Preview a patch release (0.5.0 → 0.5.1)
cargo release patch --dry-run

# Execute a patch release (updates all versions, publishes to crates.io, tags, pushes)
cargo release patch --execute
```

cargo-release automatically:
- Updates workspace.package.version
- Updates workspace.dependencies.nameback-core.version
- Publishes all packages in dependency order
- Creates git tags and triggers GitHub Actions release workflow

### Distribution and Installation

**macOS:**
- Homebrew formula: /Users/4n6h4x0r/src/homebrew-nameback/Formula/nameback.rb (CLI)
- Homebrew cask: /Users/4n6h4x0r/src/homebrew-nameback/Casks/nameback.rb (GUI)

**Windows:**
- MSI installer built via WiX Toolset (see /Users/4n6h4x0r/src/nameback/installer/nameback.wxs)
- Auto-installs all dependencies via Chocolatey
- CLI includes console hiding functionality for silent MSI custom actions
  - Detects MSIHANDLE environment variable (set by Windows Installer)
  - Uses Windows API (GetConsoleWindow, ShowWindow) to hide console
  - Prevents console flash during installation dependency checks

**Linux:**
- Debian package built via GitHub Actions (see /Users/4n6h4x0r/src/nameback/.github/workflows/release.yml)
- Includes both CLI and GUI binaries
- Desktop entry for GUI application

**GitHub Actions Release Workflow:**
The release workflow (`.github/workflows/release.yml`) handles:
- Building platform-specific installers (MSI, DMG, DEB)
- Generating SHA256 checksums with unique filenames to avoid collisions
- Creating SLSA build provenance attestations for supply chain security
- Publishing to GitHub Releases
- Publishing to crates.io

**Linux Build Fix:**
The workflow sets `PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH"` inline with the cargo build command to find leptonica/tesseract dependencies in multiarch locations.

**Checksum Verification:**
Platform-specific instructions documented in README.md:
- macOS: `shasum -a 256 -c checksums.txt --ignore-missing`
- Linux: `sha256sum -c checksums.txt --ignore-missing`
- Windows: PowerShell script for verification
