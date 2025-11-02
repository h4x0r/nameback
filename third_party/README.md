# Third-Party Dependencies

This directory contains attribution and licensing information for third-party dependencies bundled with or used by Nameback.

**Note**: The actual dependency installers are NOT stored in this repository. They are downloaded from official sources during installation, or as a fallback, from [GitHub Releases](https://github.com/h4x0r/nameback/releases).

## Bundled Dependencies

### Required

1. **ExifTool** - Metadata extraction tool
   - License: GPL-1.0-or-later OR Artistic-1.0-Perl
   - See: `exiftool/NOTICE`

### Optional

2. **Tesseract OCR** - Optical character recognition
   - License: Apache-2.0
   - See: `tesseract/NOTICE`

3. **FFmpeg** - Multimedia framework
   - License: LGPL-2.1-or-later (default build)
   - See: `ffmpeg/NOTICE`

4. **ImageMagick** - Image processing library
   - License: ImageMagick License (Apache-like)
   - See: `imagemagick/NOTICE`

## License Texts

Full license texts for all third-party dependencies are located in the `/LICENSES` directory at the repository root, following the [REUSE specification](https://reuse.software/).

## Source Code Availability

For dependencies under copyleft licenses (GPL, LGPL), source code is available:

- **ExifTool**: https://exiftool.org/
- **FFmpeg**: Source code is bundled with releases (LGPL compliance) or available at https://ffmpeg.org/download.html

## Installation

Dependencies are installed automatically during Nameback setup via:
1. System package managers (Scoop, Chocolatey, Homebrew, apt, etc.)
2. If package managers fail, bundled installers are downloaded from GitHub Releases

See the main [README.md](/README.md) for installation instructions.
