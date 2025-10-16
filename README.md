# nameback

Rename files based on their metadata. Automatically extracts titles, dates, and descriptions from your files to give them meaningful names.

## What it does

Scans a folder and renames files using metadata embedded in the files themselves:

- **Photos** → Renamed by date taken
- **PDFs** → Renamed by document title
- **Videos** → Renamed by creation date or title
- **HTML files** → Renamed by page title

Before: `IMG_2847.jpg`, `document.pdf`, `f2577888.html`
After: `2024-03-15_sunset.jpg`, `Annual_Report_2024.pdf`, `Important_safety_information.html`

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

- **Rust** - [Install from rust-lang.org](https://www.rust-lang.org/tools/install) (for installation)
- **exiftool** - `brew install exiftool` (macOS) or `apt install libimage-exiftool-perl` (Linux)

## Options

```bash
--dry-run      # Preview changes before applying
--verbose      # Show detailed progress
--skip-hidden  # Skip hidden files (like .DS_Store)
```

## Safety Features

✓ **Preview mode** - Always test with `--dry-run` first
✓ **No overwrites** - Skips files if destination already exists
✓ **Duplicate handling** - Adds `_1`, `_2` suffixes automatically
✓ **Filename sanitization** - Removes special characters safely

## Security

**nameback operates with strong security constraints:**

- **Same-directory only** - Files can only be renamed within their parent directory. No path traversal or directory changes.
- **Permission-based** - Respects standard Unix/filesystem permissions. Cannot modify files you don't have permission to change.
- **No privilege escalation** - Cannot access system directories (like `/etc/`) without appropriate permissions.
- **Overwrite protection** - Will never overwrite existing files, preventing accidental data loss.

**Best Practices:**
- Always run `--dry-run` first to preview changes
- Never run as root on system directories (`/etc`, `/usr`, etc.)
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
- **Office documents** - DOCX, XLSX, PPTX with title or author
- **OpenDocument** - ODT, ODS, ODP with title or author
- **Videos** - MP4, AVI with creation date or title
- **HTML files** - Files with title tags
- **PDFs** - PDFs with Title metadata (⚠️ see limitation below)

Files **without useful metadata** are skipped:
- **Scanned PDFs** - Only have scanner/printer names (e.g., "Canon iPR C165")
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
