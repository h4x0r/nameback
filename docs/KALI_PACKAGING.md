# Kali Linux Package Inclusion Request

This document provides the template and justification for requesting nameback's inclusion in the Kali Linux official repositories.

## Package Information

**Package Name:** nameback
**Version:** 0.4.1
**License:** MIT
**Homepage:** https://github.com/h4x0r/nameback
**Source Repository:** https://github.com/h4x0r/nameback.git
**Maintainer:** Albert Hui <albert@example.com>
**Category:** Forensics / DFIR Tools

## Short Description

Intelligent file renaming tool for digital forensics and incident response workflows, particularly useful for organizing recovered files from PhotoRec/TestDisk.

## Detailed Description

nameback is a specialized file renaming utility that extracts embedded metadata from files to generate meaningful, descriptive filenames. It supports a wide range of file types including images, videos, PDFs, documents, and more, making it an essential tool for DFIR workflows.

### Key Features

- **Smart Metadata Extraction** - EXIF, PDF metadata, document properties
- **Multi-Language OCR** - Supports 160+ languages including CJK (Chinese, Japanese, Korean)
- **Multi-Frame Video Analysis** - Extracts multiple frames for better OCR accuracy
- **HEIC/HEIF Support** - Native support for Apple's image formats
- **Series Detection** - Automatically maintains file numbering in sequences
- **Safe Operation** - Dry-run mode, no overwrites, blocks root execution
- **Dual Interface** - Both CLI and GUI available

### Technical Specifications

- **Language:** Rust
- **Dependencies:**
  - `libimage-exiftool-perl` (required)
  - `tesseract-ocr` (for OCR functionality)
  - `tesseract-ocr-chi-tra` (Traditional Chinese)
  - `tesseract-ocr-chi-sim` (Simplified Chinese)
  - `ffmpeg` (for video frame extraction)
  - `imagemagick` (for HEIC/HEIF support)
- **Binary Size:** ~15MB (CLI + GUI combined)
- **Architecture:** amd64 (can be built for arm64 if needed)

## Forensics/DFIR Use Cases

### 1. PhotoRec/TestDisk Integration

nameback is specifically designed to complement Kali's existing data recovery tools:

```bash
# Typical DFIR workflow
sudo photorec /dev/sdb1          # Recover files (generates IMG_0001.jpg, f000001.pdf, etc.)
nameback /path/to/recovered --dry-run    # Preview intelligent renames
nameback /path/to/recovered               # Apply renames based on metadata
```

**Value:** PhotoRec recovers files with generic names like `IMG_0001.jpg`, `f000001.pdf`. nameback transforms these into meaningful names like `2024-03-15_Evidence_Photo.jpg`, `Financial_Report_Q4_2024.pdf`, making evidence organization and timeline analysis significantly easier.

### 2. Evidence Timeline Analysis

Extract and organize files by their actual creation dates from EXIF metadata:

```bash
nameback --include-timestamp ~/evidence/photos
# Renames: IMG_5847.jpg → 2024-03-15_10-23-45_Crime_Scene.jpg
```

**Value:** Critical for establishing timelines in forensic investigations. EXIF timestamps are more reliable than filesystem metadata which can be modified.

### 3. Screenshot OCR for Incident Response

Process screenshots from compromised systems to extract text for search and indexing:

```bash
nameback ~/case-files/screenshots --verbose
# Uses Tesseract OCR to extract text and rename files
# screenshot_001.png → Login_Panel_Admin_Access.png
```

**Value:** Incident responders often deal with hundreds of screenshots. OCR-based renaming makes them instantly searchable and identifiable.

### 4. Multi-Language Evidence Processing

Support for international investigations with CJK language OCR:

```bash
nameback ~/evidence/chinese-docs
# Extracts Chinese characters from screenshots and documents
# screenshot_20241015.png → 輸入_姓名.png
```

**Value:** Essential for investigations involving international actors or non-English evidence.

### 5. Seized Device Media Organization

Process large volumes of media files from seized devices:

```bash
nameback /mnt/seized-phone/DCIM --include-location
# Organizes photos with GPS data: IMG_3847.heic → 2024-06-10_37.7749N_122.4194W_Sunset.heic
```

**Value:** GPS coordinates embedded in filenames help establish location intelligence without opening each file.

## Why Kali Linux?

### Perfect Fit for Kali's Mission

1. **DFIR Focus** - nameback is specifically designed for forensics workflows
2. **Complements Existing Tools** - Works perfectly with PhotoRec, TestDisk, Autopsy
3. **Incident Response** - Screenshot OCR and timeline analysis are IR staples
4. **International Investigations** - Multi-language OCR support (CJK languages)
5. **Safe by Design** - Blocks root execution, dry-run mode prevents accidental data loss

### Integration with Existing Kali Tools

- **testdisk/photorec** - Post-recovery file organization
- **autopsy** - Pre-processing for case file imports
- **bulk_extractor** - Metadata extraction complement
- **exiftool** - Extends functionality with intelligent renaming

### Target Audience

- Digital forensics professionals
- Incident responders
- Law enforcement cybercrime units
- Penetration testers (report organization)
- Security researchers

## Installation and Testing

### Pre-built Debian Package

A production-ready `.deb` package is available:

```bash
wget https://github.com/h4x0r/nameback/releases/latest/download/nameback_0.4.1-1_amd64.deb
sudo dpkg -i nameback_0.4.1-1_amd64.deb
sudo apt-get install -f  # Install dependencies
```

### Build from Source

```bash
git clone https://github.com/h4x0r/nameback.git
cd nameback
cargo build --release --workspace

# Binaries in target/release:
# - nameback (CLI)
# - nameback-gui (GUI)
```

### Debian Packaging Files

Complete Debian packaging is available in the `debian/` directory:
- `debian/control` - Package metadata and dependencies
- `debian/rules` - Build instructions
- `debian/changelog` - Version history
- `debian/copyright` - License information
- `debian/nameback.desktop` - GUI desktop entry

### Lintian Compliance

The package is designed to be lintian-clean:

```bash
lintian --no-tag-display-limit nameback_0.4.1-1_amd64.deb
# Expected: No errors, minimal warnings (if any)
```

## Quality Assurance

### Security Considerations

- **No Root Execution** - Explicitly blocks running as root to prevent accidental system file modification
- **Read-Only by Default** - Dry-run mode allows preview before any changes
- **No Network Access** - Entirely offline tool, no telemetry or phoning home
- **Rust Memory Safety** - Written in Rust for memory safety guarantees
- **Dependency Security** - Uses well-vetted dependencies (exiftool, tesseract, ffmpeg)

### Testing

Comprehensive test suite included:

```bash
cargo test --workspace
# 97 tests covering:
# - Metadata extraction
# - Filename generation
# - OCR processing
# - Series detection
# - Safety checks
```

### Documentation

- [README.md](../README.md) - Quick start and features
- [GUIDE.md](GUIDE.md) - Complete user guide
- [naming-heuristics.md](naming-heuristics.md) - Technical deep dive into intelligent naming

## Maintenance Commitment

- **Active Development** - Regular updates and feature additions
- **Responsive Maintainer** - Issues and PRs addressed promptly
- **Security Updates** - Rust's dependency update system (Renovate bot enabled)
- **Long-term Support** - Commitment to maintain for Kali inclusion

## Package Maintenance for Kali

- **Debian Native** - Package follows Debian Policy Manual
- **Reproducible Builds** - Builds are reproducible across systems
- **CI/CD Integration** - GitHub Actions automatically builds .deb packages on releases
- **Version Tracking** - Semantic versioning followed strictly

## Submission to Kali Bug Tracker

### Bug Tracker Details

**URL:** https://bugs.kali.org/
**Component:** kali-dev
**Type:** Request For Package (RFP)
**Title:** RFP: nameback - Intelligent file renaming for DFIR workflows

### Suggested Bug Description

```
Package: nameback
Version: 0.4.1
Section: forensics
Priority: optional
Architecture: amd64

Description:
 Intelligent file renaming tool based on metadata extraction, specifically
 designed for digital forensics and incident response (DFIR) workflows.

 Key features:
  * Complements PhotoRec/TestDisk by giving recovered files meaningful names
  * Multi-language OCR (160+ languages including CJK)
  * EXIF metadata extraction for timeline analysis
  * Multi-frame video analysis for better OCR accuracy
  * Safe operation with dry-run mode and root execution blocking
  * Both CLI and GUI interfaces

 Forensics use cases:
  * Organizing files recovered from data recovery operations
  * Evidence timeline reconstruction from EXIF timestamps
  * Screenshot OCR for incident response documentation
  * Multi-language evidence processing
  * Seized device media organization with GPS data

 Homepage: https://github.com/h4x0r/nameback
 Source: https://github.com/h4x0r/nameback.git
 License: MIT
 Maintainer: Albert Hui <albert@example.com>

 Pre-built .deb package available:
 https://github.com/h4x0r/nameback/releases/latest/download/nameback_0.4.1-1_amd64.deb

 Debian packaging files available in debian/ directory of source repository.
```

## References

- **Source Repository:** https://github.com/h4x0r/nameback
- **Release Downloads:** https://github.com/h4x0r/nameback/releases
- **Issue Tracker:** https://github.com/h4x0r/nameback/issues
- **Documentation:** https://github.com/h4x0r/nameback/tree/main/docs

## Contact

**Maintainer:** Albert Hui
**Email:** albert@example.com
**GitHub:** @h4x0r
**Response Time:** Typically within 24-48 hours for critical issues

---

## Next Steps

1. Submit RFP to Kali Bug Tracker: https://bugs.kali.org/
2. Monitor bug tracker for Kali team feedback
3. Respond promptly to any packaging or technical questions
4. If accepted, package will be added to kali-rolling repository
5. Announce to forensics community once available in Kali

## Additional Notes for Kali Team

- Package is ready for immediate integration
- No special build requirements beyond standard Rust toolchain
- Dependencies are already in Kali repositories
- Desktop integration tested on XFCE (Kali's default)
- No conflicts with existing packages
- Can provide additional documentation or changes as needed

Thank you for considering nameback for inclusion in Kali Linux!
