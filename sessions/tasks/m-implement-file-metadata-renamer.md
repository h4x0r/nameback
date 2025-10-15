---
name: m-implement-file-metadata-renamer
branch: feature/m-implement-file-metadata-renamer
status: pending
created: 2025-10-15
---

# File Metadata Renamer

## Problem/Goal
Create a utility that recursively scans a folder and intelligently renames files based on their metadata. For each file, the tool should:
- Determine the file type using the `file` command
- Extract metadata using `exiftool`
- Find suitable filenames from either the original filename or metadata fields
- Rename the file to a more descriptive/appropriate name

## Success Criteria
- [ ] Script successfully scans directories recursively and processes all files
- [ ] File type detection works correctly using the `file` command
- [ ] Metadata extraction works via `exiftool` for supported file types
- [ ] Files are renamed based on intelligent selection from filename or metadata fields
- [ ] Script handles edge cases (duplicate names, special characters, permission errors)
- [ ] Dry-run mode available to preview changes before applying
- [ ] Clear logging of what files were renamed and why
- [ ] Successfully tested on `/tmp/photorec` directory

## Context Manifest

### Project Overview
This is a greenfield utility project - creating a new file metadata renamer from scratch.

### Project Structure
Recommended location: `/Users/4n6h4x0r/src/nameback/nameback.py` (or similar name)
- Standalone Python script
- Can evolve into a package if needed

### Technical Requirements

**External Dependencies:**
- `file` command: System utility for file type detection
  - Available on macOS/Linux by default
  - Usage: `file -b <filepath>` for brief output

- `exiftool`: Metadata extraction utility
  - Installation: `brew install exiftool` (macOS) or `apt-get install libimage-exiftool-perl` (Linux)
  - Usage: `exiftool -json <filepath>` for structured output
  - Supports: images (JPEG, PNG, etc.), documents (PDF), audio/video files

**Python Libraries:**
- `subprocess`: Run external commands (file, exiftool)
- `pathlib`: Modern path handling
- `argparse`: Command-line argument parsing
- `json`: Parse exiftool JSON output
- `logging`: Track operations and errors

### Implementation Approach

**Core Components:**

1. **File Scanner**
   - Recursively traverse directory tree
   - Skip hidden files/directories (optional flag)
   - Handle symlinks carefully

2. **Type Detector**
   - Execute `file` command
   - Parse output to determine file category
   - Map to renaming strategies

3. **Metadata Extractor**
   - Execute `exiftool` with JSON output
   - Parse metadata fields
   - Priority fields by file type:
     - Images: DateTimeOriginal, Title, Description
     - Documents: Title, Subject, Author, CreationDate
     - Audio: Title, Artist, Album
     - Video: Title, CreationDate

4. **Filename Generator**
   - Extract potential names from metadata
   - Sanitize filenames (remove special chars, limit length)
   - Handle duplicates (append counter)
   - Preserve file extensions

5. **Renaming Engine**
   - Dry-run mode (preview only)
   - Actual rename with error handling
   - Logging of all operations

### Edge Cases & Considerations

**Duplicate Handling:**
- If target filename exists, append counter: `filename_1.ext`, `filename_2.ext`
- Preserve original if no suitable metadata found

**Special Characters:**
- Sanitize: `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`
- Replace spaces with underscores or hyphens (configurable)
- Handle Unicode characters appropriately

**Permissions & Errors:**
- Check write permissions before renaming
- Handle permission denied gracefully
- Skip files that can't be renamed, log the issue

**Dry-Run Mode:**
- Essential for testing
- Show: `old_name.ext -> new_name.ext`
- No actual file system changes

**Logging:**
- Log level: INFO for renames, WARNING for skips, ERROR for failures
- Output to stdout and optional log file
- Include: timestamp, old name, new name, reason

### Test Environment
- Primary test directory: `/tmp/photorec`
- Likely contains recovered files with generic names
- Good test case for metadata-based renaming

## User Notes
<!-- Any specific notes or requirements from the developer -->

## Work Log
<!-- Updated as work progresses -->
- [2025-10-15] Created task
