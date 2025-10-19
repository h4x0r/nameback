# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

## ✨ Key Features

- 📸 **Smart Photo Renaming** - Uses EXIF data (date, description) from JPEG, PNG, HEIC/HEIF
- 📄 **PDF Intelligence** - Extracts titles from metadata or document content, with OCR for scanned PDFs
- 🎥 **Video Processing** - Renames by metadata or extracts frames for OCR when metadata is missing
- 🌏 **Multi-Language OCR** - Supports Traditional Chinese, Simplified Chinese, English (and 160+ more languages)
- 🍎 **HEIC Support** - Native support for Apple's High Efficiency Image Format
- 🔒 **Safe & Secure** - Preview mode, no overwrites, blocks root execution, same-directory only

## What it does

Transforms meaningless filenames into descriptive ones using embedded metadata:

```
IMG_2847.jpg           → 2024-03-15_sunset.jpg
document.pdf           → Annual_Report_2024.pdf
f2577888.html          → Important_safety_information.html
screenshot_20241015.png → 輸入_姓名.png (Chinese OCR)
VID_20241015.mp4       → Product_Demo_Video.mp4 (frame OCR)
IMG_3847.heic          → Family_Reunion_2024.heic
```

## Installation

### macOS (Recommended)

**Homebrew** - One command installs everything:
```bash
brew tap h4x0r/nameback
brew install nameback
```
✅ Automatically installs all dependencies (exiftool, tesseract, ffmpeg, imagemagick)

### Windows

**1. Download binary** from [latest release](https://github.com/h4x0r/nameback/releases/latest):
   - Download `nameback-x86_64-pc-windows-msvc.exe`
   - Rename to `nameback.exe` and add to PATH

**2. Install dependencies:**
```powershell
nameback --install-deps
```
Uses winget or Chocolatey to install exiftool, tesseract, ffmpeg, imagemagick

### Linux

```bash
cargo install nameback
nameback --install-deps
```
Auto-detects your package manager (apt, dnf, yum, pacman, zypper)

## Quick Start

```bash
# Check dependencies are installed
nameback --check-deps

# Preview changes (safe, no files modified)
nameback /path/to/folder --dry-run

# Actually rename the files
nameback /path/to/folder
```

<details>
<summary><b>Alternative Installation Methods</b></summary>

### macOS - Direct Binary Download

**Intel:**
```bash
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback-x86_64-apple-darwin
chmod +x nameback-x86_64-apple-darwin
sudo mv nameback-x86_64-apple-darwin /usr/local/bin/nameback
brew install exiftool tesseract tesseract-lang ffmpeg imagemagick
```

**Apple Silicon:**
```bash
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback-aarch64-apple-darwin
chmod +x nameback-aarch64-apple-darwin
sudo mv nameback-aarch64-apple-darwin /usr/local/bin/nameback
brew install exiftool tesseract tesseract-lang ffmpeg imagemagick
```

### Windows - Manual Dependency Installation

```powershell
# With winget
winget install OliverBetz.ExifTool UB-Mannheim.TesseractOCR Gyan.FFmpeg ImageMagick.ImageMagick

# With Chocolatey
choco install exiftool tesseract ffmpeg imagemagick
```

### Linux - Manual Dependency Installation

```bash
# Debian/Ubuntu
sudo apt-get install libimage-exiftool-perl tesseract-ocr tesseract-ocr-chi-tra tesseract-ocr-chi-sim ffmpeg imagemagick

# Fedora/RHEL
sudo dnf install perl-Image-ExifTool tesseract tesseract-langpack-chi_tra tesseract-langpack-chi_sim ffmpeg imagemagick

# Arch Linux
sudo pacman -S perl-image-exiftool tesseract tesseract-data-chi_tra tesseract-data-chi_sim ffmpeg imagemagick
```

### Build from Source

```bash
git clone https://github.com/h4x0r/nameback.git
cd nameback
cargo build --release
./target/release/nameback /path/to/folder
```

</details>

## Dependency Overview

**Required:**
- **exiftool** - Core metadata extraction (always needed)

**Optional (installed automatically with `--install-deps`):**
- **tesseract-ocr** - Enables OCR for images/videos without metadata
- **ffmpeg** - Enables video frame extraction for OCR
- **ImageMagick** - Enables HEIC/HEIF support on Windows/Linux (macOS has native support via `sips`)

Check what's installed:
```bash
nameback --check-deps
```

If optional tools are installed, nameback will automatically use:
- **OCR** to extract text from:
  - Screenshots and images without EXIF metadata (supports Traditional Chinese, Simplified Chinese, and English)
  - Scanned PDFs that don't have embedded text
  - HEIC/HEIF images (Apple's High Efficiency Image Format)
  - Video frames (extracts a frame and runs OCR on it)
- **HEIC conversion** for Apple's HEIC/HEIF image format
- **Video frame extraction** for videos without metadata

## Options

```bash
--check-deps   # Check dependency installation status
--install-deps # Install missing dependencies interactively
--dry-run      # Preview changes before applying
--verbose      # Show detailed progress
--skip-hidden  # Skip hidden files (like .DS_Store)
```

## Safety & Security

**Operational Safety:**
- ✓ **Preview mode** - Always test with `--dry-run` first
- ✓ **No overwrites** - Skips files if destination already exists
- ✓ **Duplicate handling** - Adds `_1`, `_2` suffixes automatically
- ✓ **Filename sanitization** - Removes special characters safely

**Security Constraints:**
- ✓ **Same-directory only** - Files can only be renamed within their parent directory (no path traversal)
- ✓ **Permission-based** - Respects standard Unix/filesystem permissions
- ✓ **No privilege escalation** - Cannot access system directories without appropriate permissions
- ✓ **Root execution blocked** - Refuses to run as root to prevent accidental system directory modification

**Best Practices:**
- Always run `--dry-run` first to preview changes
- Use on user data directories where you have write permissions
- Back up important files before bulk renaming operations

## Examples

### Basic Usage
```bash
# Preview changes in your Photos folder (always start with dry-run!)
nameback ~/Pictures --dry-run

# Actually rename the files
nameback ~/Pictures

# Process with detailed logging to see what's happening
nameback ~/Documents --verbose
```

### Common Use Cases
```bash
# Rename recovered files from PhotoRec (data recovery)
nameback /tmp/photorec --dry-run

# Process screenshots folder with OCR
nameback ~/Desktop/Screenshots --verbose

# Organize downloaded files
nameback ~/Downloads --dry-run

# Clean up iPhone photo exports (HEIC support)
nameback ~/Desktop/iPhone_Export
```

### Real-World Example Output
```
[INFO] Processing 150 files...
[INFO] [DRY RUN] ./IMG_2847.jpg -> 2024-03-15_sunset.jpg
[INFO] [DRY RUN] ./screenshot.png -> Error_Database_Connection_Failed.png (OCR)
[INFO] [DRY RUN] ./resume.png -> 輸入_姓名.png (OCR: Traditional Chinese)
[INFO] [DRY RUN] ./scanned_doc.pdf -> Annual_Report_2024.pdf (OCR)
[INFO] [DRY RUN] ./video.mp4 -> Product_Demo_Opening_Scene.mp4 (frame OCR)
[WARN] No suitable metadata found for: random_file.dll. Skipping.
[INFO] Processing complete!
```

## Building from Source

If you want to build from source instead of installing from crates.io:

```bash
git clone https://github.com/h4x0r/nameback.git
cd nameback
cargo build --release
./target/release/nameback /path/to/folder
```

## How it works

1. **Scans** - Recursively walks through directories
2. **Detects** - Identifies file types (image, document, video, audio)
3. **Extracts** - Pulls metadata using exiftool
4. **Enhances** - Falls back to OCR for files without metadata (when tesseract is available)
5. **Generates** - Creates clean, descriptive filenames
6. **Renames** - Updates files (or shows preview in dry-run mode)

## Advanced Features

### Multi-Language OCR
When tesseract-lang is installed, nameback automatically tries multiple languages and selects the best result:
- **Traditional Chinese** (`chi_tra`) - Tried first for broader compatibility
- **Simplified Chinese** (`chi_sim`) - Second priority
- **English** (`eng`) - Always available as fallback
- **160+ additional languages** available via tesseract-lang

The system automatically selects the language that produces the most characters, ensuring optimal text extraction.

### HEIC/HEIF Support
Apple's High Efficiency Image Format is fully supported:
- Uses **sips** (macOS built-in) for fast conversion
- Falls back to **ImageMagick** if sips is unavailable
- Extracts EXIF metadata before OCR
- Maintains original .heic extension after renaming

### Video Frame OCR
For videos without useful metadata:
- Extracts frame at **1 second** using ffmpeg
- Runs multi-language OCR on the frame
- Useful for screen recordings, tutorials, presentations
- Supports: MP4, MOV, AVI, MKV, WebM, FLV, WMV, M4V

## Metadata Extraction Logic

nameback follows a smart priority system for extracting filenames. Each file type has a specific order of metadata fields to try, with intelligent fallbacks when metadata is missing or unhelpful.

### Metadata Field Priority by File Type

#### Images (JPEG, PNG, GIF, BMP, TIFF, WebP, HEIC/HEIF)
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

**Example:**
```
Photo has Title "Sunset Beach" → Uses "Sunset_Beach.jpg"
Photo has no Title but Description "Family reunion" → Uses "Family_reunion.jpg"
Screenshot has no metadata → OCR extracts "Error: Database Failed" → Uses "Error_Database_Failed.png"
```

#### PDFs
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

**Example:**
```
PDF has Title "Annual Report 2024" → Uses "Annual_Report_2024.pdf"
PDF has no Title but body text starts with "Executive Summary..." → Uses "Executive_Summary.pdf"
Scanned PDF has no text → OCR extracts "Invoice #12345" → Uses "Invoice_12345.pdf"
```

#### Videos (MP4, MOV, AVI, MKV, WebM, FLV, WMV, M4V)
**Priority Order:**
1. **Title** - Video metadata Title tag
2. **CreationDate** - Video creation timestamp
3. **Fallback:** Frame extraction + OCR
   - Extracts frame at 1 second using `ffmpeg`
   - Runs multi-language OCR (chi_tra → chi_sim → eng)
   - Uses first 80 characters

**Example:**
```
Video has Title "Product Demo" → Uses "Product_Demo.mp4"
Video has CreationDate "2024-10-15" → Uses "2024-10-15.mp4"
Screen recording has no metadata → OCR extracts "Welcome to Tutorial" → Uses "Welcome_to_Tutorial.mp4"
```

#### Office Documents (DOCX, XLSX, PPTX, ODT, ODS, ODP)
**Priority Order:**
1. **Title** - Document Title property
2. **Subject** - Document Subject property
3. **Author** - Document Author (filtered for scanner/printer names)
4. **No fallback** - Documents without useful metadata are skipped

**Filters Applied:**
- Scanner/printer names removed: "Canon", "iPR", "Printer", "Scanner"
- "Untitled" documents skipped
- Minimum 3 characters required

**Example:**
```
DOCX has Title "Q4 Sales Report" → Uses "Q4_Sales_Report.docx"
XLSX has Subject "Budget Planning" → Uses "Budget_Planning.xlsx"
PPTX has Author "Canon iPR C165" → Filtered out, checks other fields or skips
```

#### Audio Files (MP3, WAV, FLAC, OGG, M4A)
**Priority Order:**
1. **Title** - Audio Title tag (ID3)
2. **Artist** - Artist name
3. **Album** - Album name
4. **No fallback** - Audio files without useful metadata are skipped

**Example:**
```
MP3 has Title "Bohemian Rhapsody" → Uses "Bohemian_Rhapsody.mp3"
MP3 has no Title but Artist "Queen" → Uses "Queen.mp3"
WAV has no metadata → Skipped (remains unnamed)
```

### Text Extraction and OCR Details

#### PDF Text Extraction
When extracting text from PDF files:
- Uses `pdf-extract` library to read embedded text from PDF body
- Cleans whitespace, removes newlines, collapses multiple spaces
- Takes **first 80 characters** as filename
- Requires **minimum 10 characters** to be considered useful
- Filters out unhelpful text: scanner/printer names (Canon, iPR, etc.)

#### OCR Processing
When running OCR on images, PDFs, or video frames:
- **Language Detection:** Tries multiple languages sequentially
  1. Traditional Chinese (`chi_tra`)
  2. Simplified Chinese (`chi_sim`)
  3. English (`eng`)
- **Best Result Selection:** Keeps the language that extracted the most characters
- **Text Cleaning:**
  - Removes excess whitespace and newlines
  - Collapses multiple spaces into single space
  - Truncates to **first 80 characters**
  - Requires **minimum 10 characters** to be useful
- **Image Conversion (when needed):**
  - HEIC → PNG using `sips` (macOS) or ImageMagick
  - PDF → PNG using `pdftoppm` from poppler-utils
  - Video → Frame at 1 second using `ffmpeg`

#### Metadata Filtering
Certain metadata values are considered unhelpful and filtered out:
- Scanner/printer names: "Canon", "iPR", "Printer", "Scanner"
- Generic text: "Untitled"
- Too short: less than 3 characters
- Empty strings or whitespace-only

When metadata is filtered out, nameback moves to the next priority field or fallback method.

## What gets renamed?

Files **with useful metadata** get renamed:
- **Photos** - JPEG, PNG, HEIC/HEIF with EXIF data (date taken, description)
- **Screenshots/Images without EXIF** - Automatically uses OCR if tesseract is installed (optional, supports multi-language)
- **HEIC/HEIF Images** - Apple's High Efficiency Image Format, with EXIF or OCR fallback
- **Office documents** - DOCX, XLSX, PPTX with title or author
- **OpenDocument** - ODT, ODS, ODP with title or author
- **Videos** - MP4, MOV, AVI, MKV with creation date or title
- **Videos without metadata** - Extracts a frame and uses OCR if ffmpeg and tesseract are installed (optional)
- **HTML files** - Files with title tags
- **PDFs** - Uses Title metadata when available, falls back to extracting text content from the document
- **Scanned PDFs** - Automatically uses OCR if tesseract and poppler are installed (optional)

Files **without useful metadata** are skipped:
- **PDFs with only scanner names** - Metadata like "Canon iPR C165" is filtered out
- **Plain text** - TXT, CSV, MD files have no embedded metadata
- **System files** - DLLs, temp files, executables
- **Recovered files** - Files that lost metadata during recovery

## Troubleshooting

**No files renamed?**
- Run with `--verbose` to see why files are skipped
- Many files simply lack useful metadata

**Permission denied?**
- Ensure you have write access to the directory
- Try running with appropriate permissions

**Want to undo?**
- nameback doesn't keep backups - use version control
- Always test with `--dry-run` first
- Consider backing up important files before renaming

## License

MIT License - see [LICENSE](LICENSE) file for details

Created by Albert Hui ([@4n6h4x0r](https://github.com/h4x0r))

Built with [Claude Code](https://claude.com/claude-code)
