#!/bin/bash
#
# Prepare bundled Windows dependencies (portable versions)
#
# Creates a single deps-windows.zip containing all portable dependency executables
# for inclusion in the MSI installer. No Scoop/Chocolatey needed.
#
# Output: deps-windows.zip ready for MSI inclusion

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMP_DIR="$(mktemp -d)"
OUTPUT_DIR="${SCRIPT_DIR}/../target/bundled-deps"
DEPS_DIR="$TEMP_DIR/deps"

mkdir -p "$OUTPUT_DIR"
mkdir -p "$DEPS_DIR"

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

    # Extract
    cd "$work_dir"
    unzip -q exiftool.zip

    # Find the exe file recursively
    local exe_file=$(find . -name "*.exe" -type f | head -n 1)
    if [ -n "$exe_file" ]; then
        # Copy to deps folder
        mkdir -p "$DEPS_DIR/exiftool"
        cp "$exe_file" "$DEPS_DIR/exiftool/exiftool.exe"
        echo "✓ ExifTool ready"
    else
        echo "ERROR: No .exe file found after extraction"
        exit 1
    fi
}

# Tesseract OCR (portable build)
prepare_tesseract() {
    echo "📦 Preparing Tesseract OCR..."
    local work_dir="$TEMP_DIR/tesseract"

    mkdir -p "$work_dir"

    # Try multiple sources for Tesseract
    local downloaded=false

    # Source 1: UB Mannheim (primary)
    echo "Trying UB Mannheim..."
    if curl -L -f --connect-timeout 30 --max-time 120 \
        "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.5.0.20241111.exe" \
        -o "$work_dir/tesseract-setup.exe" 2>/dev/null; then
        echo "✓ Downloaded from UB Mannheim"
        downloaded=true
    fi

    # Source 2: GitHub unofficial builds (fallback)
    if [ "$downloaded" = false ]; then
        echo "Trying GitHub tesseract-ocr-w64..."
        if curl -L -f --connect-timeout 30 --max-time 120 \
            "https://github.com/UB-Mannheim/tesseract/releases/download/v5.4.0.20240606/tesseract-ocr-w64-setup-5.4.0.20240606.exe" \
            -o "$work_dir/tesseract-setup.exe" 2>/dev/null; then
            echo "✓ Downloaded from GitHub"
            downloaded=true
        fi
    fi

    if [ "$downloaded" = false ]; then
        echo "⚠️  Tesseract download failed, skipping (optional dependency)"
        return
    fi

    # Extract using 7z (available on GitHub Actions)
    cd "$work_dir"
    7z x tesseract-setup.exe -o"extracted" > /dev/null 2>&1 || true

    # Find tesseract.exe
    local tesseract_exe=$(find extracted -name "tesseract.exe" -type f | head -n 1)

    if [ -n "$tesseract_exe" ]; then
        mkdir -p "$DEPS_DIR/tesseract"
        # Copy tesseract and tessdata
        cp "$tesseract_exe" "$DEPS_DIR/tesseract/"

        # Copy tessdata directory if it exists
        local tessdata_dir=$(dirname "$tesseract_exe")/tessdata
        if [ -d "$tessdata_dir" ]; then
            cp -r "$tessdata_dir" "$DEPS_DIR/tesseract/"
        fi

        echo "✓ Tesseract ready"
    else
        echo "⚠️  Tesseract extraction failed, skipping (optional dependency)"
    fi
}

# FFmpeg (portable build)
prepare_ffmpeg() {
    echo "📦 Preparing FFmpeg..."
    local ffmpeg_version="7.1"
    local ffmpeg_url="https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2024-11-04-12-55/ffmpeg-n${ffmpeg_version}-latest-win64-gpl-${ffmpeg_version}.zip"
    local work_dir="$TEMP_DIR/ffmpeg"

    mkdir -p "$work_dir"

    # Try specific version first
    echo "Trying FFmpeg ${ffmpeg_version}..."
    if curl -L -f --connect-timeout 30 --max-time 120 \
        "$ffmpeg_url" -o "$work_dir/ffmpeg.zip" 2>/dev/null; then
        echo "✓ Downloaded FFmpeg ${ffmpeg_version}"
    else
        # Fallback to known stable version
        echo "Trying FFmpeg 7.0.2 (fallback)..."
        if ! curl -L -f --connect-timeout 30 --max-time 120 \
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2024-08-19-12-52/ffmpeg-n7.0.2-latest-win64-gpl-7.0.zip" \
            -o "$work_dir/ffmpeg.zip" 2>/dev/null; then
            echo "ERROR: FFmpeg download failed"
            exit 1
        fi
    fi

    # Extract
    cd "$work_dir"
    unzip -q ffmpeg.zip

    # Find the extracted directory
    local extracted_dir=$(find . -maxdepth 1 -type d -name "ffmpeg-*" | head -n 1)

    if [ -n "$extracted_dir" ] && [ -d "$extracted_dir/bin" ]; then
        mkdir -p "$DEPS_DIR/ffmpeg"
        cp "$extracted_dir/bin/ffmpeg.exe" "$DEPS_DIR/ffmpeg/"
        cp "$extracted_dir/bin/ffprobe.exe" "$DEPS_DIR/ffmpeg/"
        echo "✓ FFmpeg ready"
    else
        echo "ERROR: FFmpeg extraction failed"
        exit 1
    fi
}

# ImageMagick (portable build)
prepare_imagemagick() {
    echo "📦 Preparing ImageMagick..."
    local work_dir="$TEMP_DIR/imagemagick"
    mkdir -p "$work_dir"

    # Try multiple sources
    echo "Trying GitHub releases..."
    if curl -L -f "https://github.com/ImageMagick/ImageMagick/releases/download/7.1.2-8/ImageMagick-7.1.2-8-portable-Q16-HDRI-x64.7z" -o "$work_dir/imagemagick.7z" 2>/dev/null; then
        if 7z x "$work_dir/imagemagick.7z" -o"$work_dir/extracted" > /dev/null 2>&1; then
            echo "✓ Downloaded from GitHub releases"
        else
            echo "⚠️  Failed to extract, trying next source..."
            rm -f "$work_dir/imagemagick.7z"
        fi
    fi

    # Check if extraction succeeded
    if [ -d "$work_dir/extracted" ] && compgen -G "$work_dir/extracted/*.exe" > /dev/null; then
        mkdir -p "$DEPS_DIR/imagemagick"
        cp "$work_dir/extracted"/*.exe "$DEPS_DIR/imagemagick/" 2>/dev/null || true
        cp "$work_dir/extracted"/*.dll "$DEPS_DIR/imagemagick/" 2>/dev/null || true
        echo "✓ ImageMagick ready"
    else
        echo "⚠️  ImageMagick download failed, skipping (optional dependency)"
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

# Create single ZIP file with all deps
cd "$DEPS_DIR"
zip -qr "$OUTPUT_DIR/deps-windows.zip" .

# Cleanup
chmod -R u+w "$TEMP_DIR" 2>/dev/null || true
rm -rf "$TEMP_DIR"

echo
echo "=== Summary ==="
echo "Created bundled dependencies package:"
ls -lh "$OUTPUT_DIR/deps-windows.zip"
echo
echo "Contents:"
unzip -l "$OUTPUT_DIR/deps-windows.zip" | head -20

echo
echo "✅ Done! Include deps-windows.zip in MSI installer."
