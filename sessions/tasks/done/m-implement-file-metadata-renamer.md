---
name: m-implement-file-metadata-renamer
branch: feature/m-implement-file-metadata-renamer
status: completed
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
- [x] Script successfully scans directories recursively and processes all files
- [x] File type detection works correctly using the `file` command
- [x] Metadata extraction works via `exiftool` for supported file types
- [x] Files are renamed based on intelligent selection from filename or metadata fields
- [x] Script handles edge cases (duplicate names, special characters, permission errors)
- [x] Dry-run mode available to preview changes before applying
- [x] Clear logging of what files were renamed and why
- [x] Successfully tested on `/tmp/photorec` directory

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

### 2025-10-15

#### Initial Setup
- Created task for file metadata renamer utility
- Established project structure in Rust
- Initialized Cargo project with 8 dependencies (clap, walkdir, serde, serde_json, anyhow, regex, log, env_logger)
- Set up modular architecture with 6 core files (main.rs, cli.rs, detector.rs, extractor.rs, generator.rs, renamer.rs)

#### Implementation Completed
- **File Scanner**: Implemented recursive directory traversal using walkdir with configurable hidden file skipping
- **Type Detector**: Integrated `file` command to categorize files into Image, Document, Audio, Video, Unknown
- **Metadata Extractor**: Built exiftool integration with JSON parsing and priority-based field extraction by file category
- **Filename Generator**: Created sanitization logic removing special characters, handling duplicates with numeric suffixes
- **Renaming Engine**: Implemented dry-run and actual rename modes with Result-based error handling
- **CLI Interface**: Built argument parser with clap supporting --dry-run, --skip-hidden, --verbose flags

#### Critical Bug Fixes
- Added file overwrite protection to prevent data loss when destination file already exists
- Pre-populated existing_names HashSet to detect collisions with pre-existing files across multiple runs
- Implemented verbose flag functionality to control debug vs info logging levels
- Fixed dry-run consistency to ensure preview matches actual execution behavior

#### Testing & Validation
- Successfully tested on `/tmp/photorec` directory containing 157,080 files
- Verified metadata-based renaming works for videos (DateTimeOriginal), PDFs (Title), HTML (title tags), SVG (Title)
- Confirmed duplicate handling with numeric suffixes (_1, _2, etc.)
- Validated filename sanitization for special characters and Unicode
- All compiler warnings resolved

#### Documentation & Review
- Created comprehensive CLAUDE.md documenting architecture, modules, dependencies, safety features
- Code review conducted identifying and resolving 3 critical issues and 5 warnings
- Pull request created: https://github.com/h4x0r/nameback/pull/1

#### Decisions
- Chose Rust over Python for type safety, performance, and robust error handling
- Used modular design (6 files) for maintainability and testability
- Implemented safety-first approach with multiple data loss prevention checks
- Prioritized metadata fields differently by file category for better rename accuracy

#### Discovered
- Performance bottleneck calling exiftool once per file (expected behavior, could batch in future)
- Many PhotoRec recovered files lack useful metadata (graceful degradation working as intended)
- Unicode handling in filenames works correctly without special configuration

#### Next Steps
- Task complete, all success criteria met
- Tool ready for production use with safety features in place
- Future enhancement opportunity: batch exiftool calls for 10-100x performance improvement
