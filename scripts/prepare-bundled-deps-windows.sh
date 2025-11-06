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
    local work_dir="$TEMP_DIR/imagemagick"
    mkdir -p "$work_dir"

    # Try multiple sources in order of reliability
    # Source 1: GitHub releases (7z format, most reliable)
    echo "Trying GitHub releases (7z)..."
    if curl -L -f "https://github.com/ImageMagick/ImageMagick/releases/download/7.1.2-8/ImageMagick-7.1.2-8-portable-Q16-HDRI-x64.7z" -o "$work_dir/imagemagick.7z" 2>/dev/null; then
        # Extract 7z file (7z is pre-installed on GitHub Actions runners)
        if 7z x "$work_dir/imagemagick.7z" -o"$work_dir" > /dev/null 2>&1; then
            echo "✓ Downloaded and extracted from GitHub releases"
        else
            echo "⚠️  Failed to extract 7z, trying next source..."
            rm -f "$work_dir/imagemagick.7z"
        fi
    fi

    # Source 2: ImageMagick download mirror (zip format)
    if [ ! -f "$work_dir"/*.exe ] && [ ! -f "$work_dir"/*/*/*.exe ]; then
        echo "⚠️  Trying ImageMagick download mirror..."
        if curl -L -f "https://download.imagemagick.org/ImageMagick/download/windows/releases/ImageMagick-7.1.2-8-portable-Q16-HDRI-x64.zip" -o "$work_dir/imagemagick.zip" 2>/dev/null; then
            cd "$work_dir"
            if unzip -q imagemagick.zip 2>/dev/null; then
                echo "✓ Downloaded and extracted from download mirror"
            else
                echo "⚠️  Failed to extract, trying next source..."
                rm -f imagemagick.zip
            fi
        fi
    fi

    # Source 3: Official archive site
    if [ ! -f "$work_dir"/*.exe ] && [ ! -f "$work_dir"/*/*/*.exe ]; then
        echo "⚠️  Trying official archive site..."
        if curl -L -f "https://imagemagick.org/archive/binaries/ImageMagick-7.1.2-8-portable-Q16-HDRI-x64.zip" -o "$work_dir/imagemagick.zip" 2>/dev/null; then
            cd "$work_dir"
            if unzip -q imagemagick.zip 2>/dev/null; then
                echo "✓ Downloaded and extracted from archive site"
            else
                echo "⚠️  Failed to extract from archive"
            fi
        fi
    fi

    # Check if we successfully got the executables
    cd "$work_dir"
    if compgen -G "*.exe" > /dev/null 2>&1 || compgen -G "*/*/*.exe" > /dev/null 2>&1; then
        # Package all exe and dll files found (handling nested directories)
        find . -name "*.exe" -o -name "*.dll" | zip -q "$OUTPUT_DIR/deps-imagemagick-windows.zip" -@
        echo "✓ Created: deps-imagemagick-windows.zip"
    else
        echo "❌ ImageMagick download failed from all sources"
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
