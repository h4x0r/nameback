# Additional Guidance

@sessions/CLAUDE.sessions.md

This file provides instructions for Claude Code for working in the cc-sessions framework.

## Project Overview

Nameback is a command-line utility that intelligently renames files based on their metadata. It recursively scans directories, detects file types, extracts metadata using external tools, and generates descriptive filenames.

## Architecture

The project is implemented in Rust as a modular CLI application with the following structure:

### Core Modules

- **main.rs** - Entry point and file scanning logic
  - Initializes logger based on verbose flag
  - Scans directories recursively using walkdir
  - Pre-populates existing filenames to prevent duplicates
  - Coordinates processing pipeline

- **cli.rs** - Command-line argument parsing using clap
  - Directory path (required positional argument)
  - Dry-run mode flag (-n, --dry-run)
  - Skip hidden files flag (-s, --skip-hidden)
  - Verbose logging flag (-v, --verbose)

- **detector.rs** - File type detection
  - Executes `file -b` command for type detection
  - Categorizes files: Image, Document, Audio, Video, Unknown
  - Pattern matching on file command output

- **extractor.rs** - Metadata extraction
  - Executes `exiftool -json` for metadata extraction
  - Parses JSON output into FileMetadata struct
  - Priority-based name extraction by file category:
    - Images: Title > Description > DateTimeOriginal
    - Documents: Title > Subject > Author
    - Audio: Title > Artist > Album
    - Video: Title > CreationDate

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
  - Supports dry-run preview mode
  - Handles errors gracefully with logging

### Dependencies

Defined in /Users/4n6h4x0r/src/nameback/Cargo.toml:
- clap 4.5 - CLI argument parsing with derive macros
- walkdir 2.4 - Recursive directory traversal
- serde 1.0 - Serialization framework
- serde_json 1.0 - JSON parsing for exiftool output
- anyhow 1.0 - Error handling with context
- regex 1.10 - Filename sanitization
- log 0.4 - Logging facade
- env_logger 0.11 - Logger implementation

### External Tool Requirements

The application depends on two external command-line tools:

- **file** - System utility for file type detection (pre-installed on macOS/Linux)
- **exiftool** - Metadata extraction tool (install via `brew install exiftool` or `apt-get install libimage-exiftool-perl`)

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

### Usage Examples

See /Users/4n6h4x0r/src/nameback/src/cli.rs for all available command-line flags.

Basic usage:
```
nameback /path/to/directory
```

Preview mode (no changes):
```
nameback --dry-run /path/to/directory
```

Skip hidden files with verbose output:
```
nameback --skip-hidden --verbose /path/to/directory
```

### Build and Run

Standard Rust project build:
```
cargo build --release
cargo run -- /path/to/directory
```

Binary location after build: /Users/4n6h4x0r/src/nameback/target/release/nameback
