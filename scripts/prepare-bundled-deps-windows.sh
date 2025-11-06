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

    # ExifTool extracts into a subdirectory (e.g., exiftool-13.41_64/)
    # Find the exe file recursively
    local exe_file=$(find . -name "*.exe" -type f | head -n 1)
    if [ -n "$exe_file" ]; then
        cp "$exe_file" exiftool.exe
        echo "Found and copied $exe_file to exiftool.exe"
    else
        echo "ERROR: No .exe file found after extraction"
        echo "Contents of extraction:"
        find . -type f
        exit 1
    fi

    # Create portable package
    zip -q "$OUTPUT_DIR/deps-exiftool-windows.zip" exiftool.exe

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
    # Use GitHub releases for more reliable downloads
    local imagemagick_url="https://github.com/ImageMagick/ImageMagick/releases/download/7.1.1-43/ImageMagick-7.1.1-43-portable-Q16-HDRI-x64.zip"
    local work_dir="$TEMP_DIR/imagemagick"

    mkdir -p "$work_dir"

    # Download with error checking
    if ! curl -L -f "$imagemagick_url" -o "$work_dir/imagemagick.zip"; then
        echo "⚠️  Failed to download ImageMagick from GitHub, trying official site..."
        imagemagick_url="https://imagemagick.org/archive/binaries/ImageMagick-7.1.1-43-portable-Q16-HDRI-x64.zip"
        if ! curl -L -f "$imagemagick_url" -o "$work_dir/imagemagick.zip"; then
            echo "❌ ImageMagick download failed from all sources"
            echo "⚠️  Skipping ImageMagick (optional dependency)"
            return 0
        fi
    fi

    # Verify it's actually a ZIP file
    if ! file "$work_dir/imagemagick.zip" | grep -q "Zip archive"; then
        echo "❌ Downloaded file is not a valid ZIP archive:"
        file "$work_dir/imagemagick.zip"
        cat "$work_dir/imagemagick.zip"
        echo "⚠️  Skipping ImageMagick (optional dependency)"
        return 0
    fi

    # Extract and package
    cd "$work_dir"
    unzip -q imagemagick.zip

    # Find and package exe/dll files
    if compgen -G "*.exe" > /dev/null; then
        zip -q -r "$OUTPUT_DIR/deps-imagemagick-windows.zip" *.exe *.dll
        echo "✓ Created: deps-imagemagick-windows.zip"
    else
        echo "❌ No exe files found after extraction"
        echo "⚠️  Skipping ImageMagick (optional dependency)"
        return 0
    fi
}

# Main execution
echo "Starting bundled dependency preparation..."
echo

# Prepare all dependencies
prepare_exiftool
prepare_tesseract
prepare_ffmpeg
prepare_imagemagick

# Cleanup (change permissions first to handle read-only files)
chmod -R u+w "$TEMP_DIR" 2>/dev/null || true
rm -rf "$TEMP_DIR"

echo
echo "=== Summary ==="
echo "All bundled dependencies created in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"/*.zip

echo
echo "✅ Done! Upload these files to GitHub Release as fallback installers."
