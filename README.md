# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

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
1. Download `nameback-x86_64-pc-windows-msvc.exe` from [releases](https://github.com/h4x0r/nameback/releases/latest)
2. Rename to `nameback.exe` and add to PATH
3. Install dependencies:
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

```bash
# Preview what will change (safe, no modifications)
nameback ~/Pictures --dry-run

# Rename the files
nameback ~/Pictures
```

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

- **Smart Photo Renaming** - Uses EXIF data (date, description) from JPEG, PNG, HEIC/HEIF
- **PDF Intelligence** - Extracts titles from metadata or document content, with OCR for scanned PDFs
- **Video Processing** - Renames by metadata or extracts frames for OCR when metadata is missing
- **Multi-Language OCR** - Supports Traditional Chinese, Simplified Chinese, English (and 160+ more languages)
- **HEIC Support** - Native support for Apple's High Efficiency Image Format
- **Safe & Secure** - Preview mode, no overwrites, blocks root execution, same-directory only

## Options

```bash
nameback <directory>              # Rename files
nameback <directory> --dry-run    # Preview changes only
nameback <directory> --verbose    # Show detailed progress
nameback <directory> --skip-hidden # Skip hidden files
nameback --check-deps             # Check dependencies
nameback --install-deps           # Install dependencies
```

## Learn More

- [Complete Guide](docs/GUIDE.md) - Installation options, advanced features, troubleshooting
- [Metadata Extraction Details](docs/GUIDE.md#metadata-extraction-details) - How nameback prioritizes different metadata fields
- [Advanced Features](docs/GUIDE.md#advanced-features) - Multi-language OCR, HEIC support, video frame extraction
- [Troubleshooting](docs/GUIDE.md#troubleshooting) - Common issues and solutions

## License

MIT License - see [LICENSE](LICENSE) file for details

Created by Albert Hui ([@4n6h4x0r](https://github.com/h4x0r))

Built with [Claude Code](https://claude.com/claude-code)
