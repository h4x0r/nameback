# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

**Available Tools:**
- 🖥️ **CLI** - Command-line tool for automation and scripting
- 🎨 **GUI** - Visual dual-pane interface (Midnight Commander style) for Windows

## What it does

Transforms meaningless filenames into descriptive ones using embedded metadata:

```
IMG_2847.jpg           → 2024-03-15_sunset.jpg
document.pdf           → Annual_Report_2024.pdf
screenshot_20241015.png → 輸入_姓名.png (Chinese OCR)
VID_20241015.mp4       → Product_Demo_Video.mp4 (frame OCR)
IMG_3847.heic          → Family_Reunion_2024.heic
```

## Installation

### macOS
```bash
brew tap h4x0r/nameback
brew install nameback
```
Automatically installs all dependencies (exiftool, tesseract, ffmpeg, imagemagick).

### Windows

**Option 1: MSI Installer (Recommended)**

Download and install `nameback-x86_64-pc-windows-msvc.msi` from [releases](https://github.com/h4x0r/nameback/releases/latest)

Includes both CLI and GUI tools:
- **CLI**: Automatically added to PATH - use `nameback` in any terminal
- **GUI**: Start Menu shortcut - launch visual interface

**Option 2: Portable Executables**

Download from [releases](https://github.com/h4x0r/nameback/releases/latest):
- CLI: `nameback-x86_64-pc-windows-msvc.exe`
- GUI: `nameback-gui-x86_64-pc-windows-msvc.exe`

**Install dependencies:**
```powershell
nameback --install-deps
```

### Linux
```bash
cargo install nameback
nameback --install-deps
```

[See all installation options](docs/GUIDE.md#installation-options)

## Quick Start

### CLI (Command-line)

```bash
# Preview what will change (safe, no modifications)
nameback ~/Pictures --dry-run

# Rename the files
nameback ~/Pictures
```

### GUI (Windows)

1. Launch **nameback** from Start Menu
2. Click **"📁 Select Directory"** to choose a folder
3. Review proposed renames in the right pane (original names on left)
4. Check/uncheck files to rename
5. Click **"✅ Rename X Files"** to apply changes

**Features:**
- 📂 Visual dual-pane interface (Midnight Commander style)
- ✅ Checkbox selection for individual files
- 🔄 Real-time preview before renaming
- ✔️ Color-coded status (pending, success, error)

## Common Examples

```bash
# Organize recovered files from data recovery
nameback /tmp/photorec --dry-run

# Process screenshots folder with OCR
nameback ~/Desktop/Screenshots --verbose

# Clean up iPhone photo exports (HEIC support)
nameback ~/Desktop/iPhone_Export

# Organize downloaded documents
nameback ~/Downloads --dry-run
```

## Features

- **Intelligent Naming Heuristics** - Quality scoring system to choose the best name from multiple sources
- **Smart Photo Renaming** - Uses EXIF data (date, description, GPS location) from JPEG, PNG, HEIC/HEIF
- **PDF Intelligence** - Extracts titles from metadata or document content, with OCR for scanned PDFs
- **Enhanced Text Extraction** - Markdown frontmatter, CSV semantic columns, nested JSON/YAML fields
- **Context-Aware Naming** - Leverages directory structure and filename analysis for better names
- **Multi-Frame Video Analysis** - Extracts multiple frames (1s, 5s, 10s) and picks the best OCR result
- **Series Detection** - Automatically detects and maintains file series numbering (e.g., vacation_001.jpg, vacation_002.jpg)
- **Format-Specific Handlers** - Email files (.eml), web archives (.html), archives (.zip, .tar), source code docstrings
- **Location & Timestamp Enrichment** - Optional GPS coordinates and formatted timestamps in filenames
- **Multi-Language OCR** - Supports Traditional Chinese, Simplified Chinese, English (and 160+ more languages)
- **Advanced Filtering** - Automatically rejects low-quality names (errors, device names, generic placeholders)
- **HEIC Support** - Native support for Apple's High Efficiency Image Format
- **Safe & Secure** - Preview mode, no overwrites, blocks root execution, same-directory only

## Options

```bash
nameback <directory>                        # Rename files
nameback <directory> --dry-run              # Preview changes only
nameback <directory> --verbose              # Show detailed progress
nameback <directory> --skip-hidden          # Skip hidden files
nameback <directory> --include-location     # Add GPS coordinates to photo/video names
nameback <directory> --include-timestamp    # Add formatted timestamps to names
nameback <directory> --multiframe-video     # Use multi-frame video analysis (slower but better)
nameback --check-deps                       # Check dependencies
nameback --install-deps                     # Install dependencies
```

## Building from Source

This project uses a Cargo workspace with 3 crates:
- **`nameback-core`** - Shared library with core rename logic
- **`nameback-cli`** - Command-line tool
- **`nameback-gui`** - GUI application (egui-based)

### Build All

```bash
# Build everything
cargo build --release

# Binaries:
# target/release/nameback (CLI)
# target/release/nameback-gui (GUI)
```

### Build Individual Tools

```bash
# CLI only
cargo build --release -p nameback

# GUI only
cargo build --release -p nameback-gui

# Core library only
cargo build --release -p nameback-core
```

### Running Tests

```bash
cargo test --workspace
```

## Learn More

- [Complete Guide](docs/GUIDE.md) - Installation options, advanced features, troubleshooting
- **[Intelligent Naming Heuristics](docs/naming-heuristics.md)** - How nameback chooses the best filename using quality scoring, content extraction, and smart filtering
- [Metadata Extraction Details](docs/GUIDE.md#metadata-extraction-details) - How nameback prioritizes different metadata fields
- [Advanced Features](docs/GUIDE.md#advanced-features) - Multi-language OCR, HEIC support, video frame extraction
- [Troubleshooting](docs/GUIDE.md#troubleshooting) - Common issues and solutions

## License

MIT License - see [LICENSE](LICENSE) file for details

Created by Albert Hui ([@4n6h4x0r](https://github.com/h4x0r))

Built with [Claude Code](https://claude.com/claude-code)
