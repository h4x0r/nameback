# Dependency Installers Directory

This directory is **NOT** included in version control and is used only for local testing of bundled installers.

## Important Notes

- ⚠️ **DO NOT commit binaries to this directory**
- ⚠️ Everything in this directory is gitignored (except this README)
- ✅ Official dependency installers are distributed via [GitHub Releases](https://github.com/h4x0r/nameback/releases)

## Purpose

This directory exists to:

1. Document the expected structure for bundled installers
2. Provide a location for local testing before uploading to GitHub Releases
3. Serve as a download cache during development

## Expected Structure (for GitHub Releases)

When bundled installers are attached to GitHub Releases, they follow this naming convention:

```
deps-exiftool-windows.zip
deps-exiftool-macos.zip
deps-exiftool-linux.tar.gz

deps-tesseract-windows.zip
deps-tesseract-macos.zip
deps-tesseract-linux.tar.gz

deps-ffmpeg-windows-lgpl.zip
deps-ffmpeg-macos-lgpl.zip
deps-ffmpeg-linux-lgpl.tar.gz
deps-ffmpeg-source.tar.gz         # Required for LGPL compliance

deps-imagemagick-windows.zip
deps-imagemagick-macos.zip
```

## Internal Structure (example)

If you download and extract these locally for testing:

```
deps/
├── exiftool/
│   ├── windows/
│   │   └── exiftool.exe
│   ├── macos/
│   │   └── exiftool
│   └── linux/
│       └── exiftool
│
├── tesseract/
│   ├── windows/
│   │   └── tesseract-installer.exe
│   ├── macos/
│   │   └── tesseract.pkg
│   └── traineddata/
│       ├── eng.traineddata
│       ├── chi_tra.traineddata
│       └── chi_sim.traineddata
│
├── ffmpeg/
│   ├── windows/
│   │   └── ffmpeg.exe
│   ├── macos/
│   │   └── ffmpeg
│   └── source/
│       └── ffmpeg-6.1.tar.gz
│
└── imagemagick/
    ├── windows/
    │   └── ImageMagick-portable.zip
    └── macos/
        └── ImageMagick.pkg
```

## License Information

All dependencies have proper licensing allowing redistribution:

- **ExifTool**: GPL-1.0-or-later OR Perl Artistic License
- **Tesseract**: Apache-2.0
- **FFmpeg**: LGPL-2.1-or-later (LGPL builds only)
- **ImageMagick**: ImageMagick License

See `/third_party/` directory for full attribution and license information.

## Automated Distribution

Dependency installers are automatically:

1. Downloaded from official sources by GitHub Actions
2. Verified with checksums
3. Packaged with proper licensing documentation
4. Uploaded to GitHub Releases

See `.github/workflows/bundle-dependencies.yml` for the automation workflow.

## For Developers

To test the bundled installer fallback locally:

1. Download installers from official sources
2. Place in appropriate subdirectories following the structure above
3. Test the `install_from_bundled_installer()` fallback in `deps.rs`
4. **DO NOT** commit - this directory is gitignored

## Questions?

See:
- `/DEPENDENCY_REDISTRIBUTION_ANALYSIS.md` - Legal analysis and implementation plan
- `/third_party/README.md` - Attribution information
- `/LICENSES/` - License texts
