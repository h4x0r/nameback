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

#### Direct Archive Download

**Intel (CLI + GUI):**
```bash
# Download and extract archive
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback-x86_64-apple-darwin.tar.gz
tar -xzf nameback-x86_64-apple-darwin.tar.gz

# Install both binaries
chmod +x nameback nameback-gui
sudo mv nameback /usr/local/bin/
sudo mv nameback-gui /usr/local/bin/

# Dependencies
brew install exiftool tesseract tesseract-lang ffmpeg imagemagick
```

**Apple Silicon (CLI + GUI):**
```bash
# Download and extract archive
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback-aarch64-apple-darwin.tar.gz
tar -xzf nameback-aarch64-apple-darwin.tar.gz

# Install both binaries
chmod +x nameback nameback-gui
sudo mv nameback /usr/local/bin/
sudo mv nameback-gui /usr/local/bin/

# Dependencies
brew install exiftool tesseract tesseract-lang ffmpeg imagemagick
```

### Windows

#### MSI Installer (Recommended)

Download and install `nameback-x86_64-pc-windows-msvc.msi` from [releases](https://github.com/h4x0r/nameback/releases/latest)

**What you get:**
- **CLI**: Type `nameback` in any terminal (automatically added to PATH)
- **GUI**: Launch from Start Menu → nameback

**Install dependencies:**
```powershell
nameback --install-deps
```

Or manually:
```powershell
# With winget
winget install OliverBetz.ExifTool UB-Mannheim.TesseractOCR Gyan.FFmpeg ImageMagick.ImageMagick

# With Chocolatey
choco install exiftool tesseract ffmpeg imagemagick
```

#### Portable Archive

For portable use without installation:
1. Download `nameback-x86_64-pc-windows-msvc.zip` from [releases](https://github.com/h4x0r/nameback/releases/latest)
2. Extract the archive to get both `nameback.exe` (CLI) and `nameback-gui.exe` (GUI)
3. Install dependencies with `nameback --install-deps`

### Linux

#### Via Cargo
```bash
cargo install nameback
nameback --install-deps
```

#### Manual Dependency Installation

**Debian/Ubuntu:**
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

```bash
git clone https://github.com/h4x0r/nameback.git
cd nameback
cargo build --release
./target/release/nameback /path/to/folder
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

Nameback follows a smart priority system for extracting filenames. Each file type has a specific order of metadata fields to try, with intelligent fallbacks when metadata is missing or unhelpful.

### Images (JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF)

**Priority Order:**
1. **Title** - EXIF Title tag
2. **Description** - EXIF Description tag
3. **DateTimeOriginal** - Date photo was taken (EXIF)
4. **Fallback:** OCR text extraction
   - Tries Traditional Chinese (`chi_tra`)
   - Then Simplified Chinese (`chi_sim`)
   - Finally English (`eng`)
   - Selects language with most extracted characters
   - Uses first 80 characters of best result

**Examples:**
```
Photo has Title "Sunset Beach" → Sunset_Beach.jpg
Photo has Description "Family reunion" → Family_reunion.jpg
Screenshot has no metadata → OCR extracts "Error: Database Failed" → Error_Database_Failed.png
```

### PDFs

**Priority Order:**
1. **Title** - PDF metadata Title field
2. **Subject** - PDF metadata Subject field
3. **Fallback:** Text content extraction
   - Extracts embedded text from PDF body
   - Uses first 80 characters
   - Requires minimum 10 characters
4. **Final Fallback:** OCR on first page
   - Converts page to PNG using `pdftoppm`
   - Runs multi-language OCR (chi_tra → chi_sim → eng)
   - Uses first 80 characters

**Examples:**
```
PDF has Title "Annual Report 2024" → Annual_Report_2024.pdf
PDF has body text "Executive Summary..." → Executive_Summary.pdf
Scanned PDF → OCR extracts "Invoice #12345" → Invoice_12345.pdf
```

### Videos (MP4, MOV, AVI, MKV, WebM, FLV, WMV, M4V)

**Priority Order:**
1. **Title** - Video metadata Title tag
2. **CreationDate** - Video creation timestamp
3. **Fallback:** Frame extraction + OCR
   - Extracts frame at 1 second using `ffmpeg`
   - Runs multi-language OCR (chi_tra → chi_sim → eng)
   - Uses first 80 characters

**Examples:**
```
Video has Title "Product Demo" → Product_Demo.mp4
Video has CreationDate "2024-10-15" → 2024-10-15.mp4
Screen recording → OCR extracts "Welcome to Tutorial" → Welcome_to_Tutorial.mp4
```

### Office Documents (DOCX, XLSX, PPTX, ODT, ODS, ODP)

**Priority Order:**
1. **Title** - Document Title property
2. **Subject** - Document Subject property
3. **Author** - Document Author (filtered for scanner/printer names)
4. **No fallback** - Documents without useful metadata are skipped

**Filters Applied:**
- Scanner/printer names removed: "Canon", "iPR", "Printer", "Scanner"
- "Untitled" documents skipped
- Minimum 3 characters required

**Examples:**
```
DOCX has Title "Q4 Sales Report" → Q4_Sales_Report.docx
XLSX has Subject "Budget Planning" → Budget_Planning.xlsx
PPTX has Author "Canon iPR C165" → Filtered out, skipped
```

### Audio Files (MP3, WAV, FLAC, OGG, M4A)

**Priority Order:**
1. **Title** - Audio Title tag (ID3)
2. **Artist** - Artist name
3. **Album** - Album name
4. **No fallback** - Audio files without useful metadata are skipped

**Examples:**
```
MP3 has Title "Bohemian Rhapsody" → Bohemian_Rhapsody.mp3
MP3 has Artist "Queen" → Queen.mp3
WAV has no metadata → Skipped
```

### OCR Processing Details

When running OCR on images, PDFs, or video frames:

**Language Detection:**
- Tries multiple languages sequentially: Traditional Chinese → Simplified Chinese → English
- Keeps the language that extracted the most characters

**Text Cleaning:**
- Removes excess whitespace and newlines
- Collapses multiple spaces into single space
- Truncates to **first 80 characters**
- Requires **minimum 10 characters** to be useful

**Image Conversion:**
- HEIC → PNG using `sips` (macOS) or ImageMagick (Windows/Linux)
- PDF → PNG using `pdftoppm` from poppler-utils
- Video → Frame at 1 second using `ffmpeg`

### Metadata Filtering

Certain metadata values are considered unhelpful and filtered out:

- Scanner/printer names: "Canon", "iPR", "Printer", "Scanner"
- Generic text: "Untitled"
- Too short: less than 3 characters
- Empty strings or whitespace-only

When metadata is filtered out, nameback moves to the next priority field or fallback method.

### What Gets Renamed?

Files **with useful metadata** get renamed:

- **Photos** - JPEG, PNG, HEIC/HEIF with EXIF data (date taken, description)
- **Screenshots/Images without EXIF** - Automatically uses OCR if tesseract is installed
- **Office documents** - DOCX, XLSX, PPTX with title or author
- **OpenDocument** - ODT, ODS, ODP with title or author
- **Videos** - MP4, MOV, AVI, MKV with creation date or title
- **Videos without metadata** - Extracts a frame and uses OCR if ffmpeg and tesseract are installed
- **HTML files** - Files with title tags
- **PDFs** - Uses Title metadata or extracts text content
- **Scanned PDFs** - Automatically uses OCR if tesseract and poppler are installed

Files **without useful metadata** are skipped:

- **PDFs with only scanner names** - Metadata like "Canon iPR C165" is filtered out
- **Plain text** - TXT, CSV, MD files have no embedded metadata
- **System files** - DLLs, temp files, executables
- **Recovered files** - Files that lost metadata during recovery

---

## Safety & Security

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
