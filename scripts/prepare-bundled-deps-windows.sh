#!/bin/bash
#
# Prepare bundled Windows dependency installers
#
# Creates ZIP files containing portable versions of dependencies
# for use as final fallback when Scoop/Chocolatey fail.
#
# Output: deps-{name}-windows.zip files ready for GitHub Release

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR="$(mktemp -d)"
OUTPUT_DIR="${SCRIPT_DIR}/../target/bundled-deps"

mkdir -p "$OUTPUT_DIR"

echo "=== Preparing Bundled Windows Dependencies ==="
echo "Temp dir: $TEMP_DIR"
echo "Output dir: $OUTPUT_DIR"
echo

# ExifTool (portable Perl version)
prepare_exiftool() {
    echo "📦 Preparing ExifTool..."
    local exiftool_version="13.41"
    local exiftool_url="https://exiftool.org/exiftool-${exiftool_version}_64.zip"
    local work_dir="$TEMP_DIR/exiftool"

    mkdir -p "$work_dir"
    curl -L "$exiftool_url" -o "$work_dir/exiftool.zip"

    # Extract and repackage
    cd "$work_dir"
    unzip -q exiftool.zip

    # Create portable package
    zip -q -r "$OUTPUT_DIR/deps-exiftool-windows.zip" exiftool.exe

    echo "✓ Created: deps-exiftool-windows.zip"
}

# Tesseract OCR (installer executable)
prepare_tesseract() {
    echo "📦 Preparing Tesseract OCR..."
    local tesseract_version="5.5.0.20241111"
    local tesseract_url="https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-${tesseract_version}.exe"
    local work_dir="$TEMP_DIR/tesseract"

    mkdir -p "$work_dir"
    curl -L "$tesseract_url" -o "$work_dir/tesseract-windows-setup.exe"

    # Package the installer
    cd "$work_dir"
    zip -q "$OUTPUT_DIR/deps-tesseract-windows.zip" tesseract-windows-setup.exe

    echo "✓ Created: deps-tesseract-windows.zip"
}

# FFmpeg (portable build)
prepare_ffmpeg() {
    echo "📦 Preparing FFmpeg..."
    local ffmpeg_url="https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
    local work_dir="$TEMP_DIR/ffmpeg"

    mkdir -p "$work_dir"
    curl -L "$ffmpeg_url" -o "$work_dir/ffmpeg.zip"

    # Extract just the binaries we need
    cd "$work_dir"
    unzip -q ffmpeg.zip

    # Find the extracted directory (name varies)
    local extracted_dir=$(find . -maxdepth 1 -type d -name "ffmpeg-*" | head -n 1)

    # Package just the executables
    cd "$extracted_dir/bin"
    zip -q "$OUTPUT_DIR/deps-ffmpeg-windows.zip" ffmpeg.exe ffprobe.exe

    echo "✓ Created: deps-ffmpeg-windows.zip"
}

# ImageMagick (portable build)
prepare_imagemagick() {
    echo "📦 Preparing ImageMagick..."
    local imagemagick_url="https://imagemagick.org/archive/binaries/ImageMagick-7.1.1-43-portable-Q16-x64.zip"
    local work_dir="$TEMP_DIR/imagemagick"

    mkdir -p "$work_dir"
    curl -L "$imagemagick_url" -o "$work_dir/imagemagick.zip"

    # Extract and package
    cd "$work_dir"
    unzip -q imagemagick.zip

    # Package the portable version
    zip -q -r "$OUTPUT_DIR/deps-imagemagick-windows.zip" *.exe *.dll

    echo "✓ Created: deps-imagemagick-windows.zip"
}

# Main execution
echo "Starting bundled dependency preparation..."
echo

# Prepare all dependencies
prepare_exiftool
prepare_tesseract
prepare_ffmpeg
prepare_imagemagick

# Cleanup
rm -rf "$TEMP_DIR"

echo
echo "=== Summary ==="
echo "All bundled dependencies created in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"/*.zip

echo
echo "✅ Done! Upload these files to GitHub Release as fallback installers."
