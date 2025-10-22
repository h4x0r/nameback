# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

**Available Tools:**
- 🖥️ **CLI** - Command-line tool for automation and scripting (Windows, macOS, Linux)
- 🎨 **GUI** - Visual dual-pane interface (Midnight Commander style) for all platforms

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

Download and install `nameback-x86_64-pc-windows-msvc.msi` from [releases](https://github.com/h4x0r/nameback/releases/latest)

**What you get:**
- **CLI**: Type `nameback` in any terminal (automatically added to PATH)
- **GUI**: Launch from Start Menu → nameback

**Alternative: Portable Archive**

Download `nameback-x86_64-pc-windows-msvc.zip` from [releases](https://github.com/h4x0r/nameback/releases/latest), extract it to get both:
- `nameback.exe` (CLI)
- `nameback-gui.exe` (GUI)

Then install dependencies:
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

### GUI (All Platforms)

**Windows:** Start Menu → nameback
**macOS/Linux (Homebrew):** Run `nameback-gui` in terminal
**macOS/Linux (portable):** Run `./nameback-gui` from download location

1. Click **"📁 Select Directory"** to choose a folder
2. Review proposed renames in the right pane (original names on left)
3. Check/uncheck files to rename
4. Click **"✅ Rename X Files"** to apply changes

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
- **Multi-Frame Video Analysis** - Extracts multiple frames (1s, 5s, 10s) and picks the best OCR result (default behavior)
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
nameback <directory> --fast-video           # Use single-frame video analysis (faster, less accurate)
nameback --check-deps                       # Check dependencies
nameback --install-deps                     # Install dependencies
```

## Learn More

- [Complete Guide](docs/GUIDE.md) - Installation options, advanced features, troubleshooting
- **[Intelligent Naming Heuristics](docs/naming-heuristics.md)** - How nameback chooses the best filename using quality scoring, content extraction, and smart filtering
- [Metadata Extraction Details](docs/GUIDE.md#metadata-extraction-details) - How nameback prioritizes different metadata fields
- [Advanced Features](docs/GUIDE.md#advanced-features) - Multi-language OCR, HEIC support, video frame extraction
- [Troubleshooting](docs/GUIDE.md#troubleshooting) - Common issues and solutions

## Security

Nameback is developed with security best practices. For automated security controls in your GitHub repositories, check out [1-click-github-sec](https://github.com/h4x0r/1-click-github-sec) by the same author (Albert Hui) - automated Dependabot, CodeQL, secret scanning, and more.

## License

MIT License - see [LICENSE](LICENSE) file for details

Created by Albert Hui ([@4n6h4x0r](https://github.com/h4x0r))

Built with [Claude Code](https://claude.com/claude-code)
