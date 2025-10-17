# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

## What it does

Scans a folder and renames files using metadata embedded in the files themselves:

- **Photos** → Renamed by date taken
- **PDFs** → Renamed by document title
- **Videos** → Renamed by creation date or title
- **HTML files** → Renamed by page title

- `IMG_2847.jpg` → `2024-03-15_sunset.jpg`
- `document.pdf` → `Annual_Report_2024.pdf`
- `f2577888.html` → `Important_safety_information.html`

## Quick Start

```bash
# Install from crates.io
cargo install nameback

# Preview what would change (safe, no files modified)
nameback /path/to/folder --dry-run

# Actually rename the files
nameback /path/to/folder
```

## Requirements

**Required:**
- **Rust** - [Install from rust-lang.org](https://www.rust-lang.org/tools/install) (for installation)
- **exiftool** - `brew install exiftool` (macOS) or `apt install libimage-exiftool-perl` (Linux)

**Optional (for OCR support):**
- **tesseract-ocr** - `brew install tesseract` (macOS) or `apt install tesseract-ocr` (Linux)
- **poppler-utils** - `brew install poppler` (macOS) or `apt install poppler-utils` (Linux)

If tesseract and poppler are installed, nameback will automatically use OCR to extract text from:
- Screenshots and images without EXIF metadata
- Scanned PDFs that don't have embedded text

## Options

```bash
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

```bash
# Preview changes in your Photos folder
nameback ~/Pictures --dry-run

# Rename recovered files from PhotoRec
nameback /tmp/photorec

# Process with detailed logging
nameback ~/Documents --verbose
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

1. Scans directory recursively
2. Detects file type (image, document, video, audio)
3. Extracts metadata using exiftool
4. Generates clean filename from title/date
5. Renames file (or shows preview in dry-run mode)

## What gets renamed?

Files **with useful metadata** get renamed:
- **Photos** - JPEG, PNG with EXIF data (date taken, description)
- **Screenshots/Images without EXIF** - Automatically uses OCR if tesseract is installed (optional)
- **Office documents** - DOCX, XLSX, PPTX with title or author
- **OpenDocument** - ODT, ODS, ODP with title or author
- **Videos** - MP4, AVI with creation date or title
- **HTML files** - Files with title tags
- **PDFs** - Uses Title metadata when available, falls back to extracting text content from the document
- **Scanned PDFs** - Automatically uses OCR if tesseract is installed (optional)

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
