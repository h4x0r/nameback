# Assets Directory

This directory contains all application assets used in builds and the application itself.

## Structure

```
assets/
├── icons/
│   ├── app-icon.png          # Main app icon (512x512)
│   ├── app-icon@2x.png       # Retina version (1024x1024)
│   └── app-icon.icns         # macOS icon bundle (TODO)
└── branding/
    └── security-ronin-logo.png  # Security Ronin branding logo (16KB)
```

## Icon Specifications

### App Icon (`icons/app-icon.png`)
- **Size**: 512x512 pixels (base), 1024x1024 pixels (retina @2x)
- **Format**: PNG with transparent background
- **Usage**:
  - macOS DMG installer
  - Application bundle Resources
  - Desktop shortcuts
  - About dialogs

### Security Ronin Logo (`branding/security-ronin-logo.png`)
- **Size**: Original dimensions preserved
- **Format**: PNG
- **Usage**:
  - GUI About dialog
  - Documentation
  - Marketing materials

## Usage in Code

### GUI Application
```rust
// Load Security Ronin logo in About dialog
let logo_bytes = include_bytes!("../../assets/branding/security-ronin-logo.png");
```

### Build Scripts (DMG)
```bash
# Copy app icon to macOS app bundle
cp assets/icons/app-icon.png "Nameback.app/Contents/Resources/"
```

## Generating Icon Variants

### Create Retina Version (@2x)
```bash
sips -z 1024 1024 assets/icons/app-icon.png --out assets/icons/app-icon@2x.png
```

### Create macOS .icns Bundle (TODO)
```bash
# Install iconutil (comes with Xcode)
mkdir AppIcon.iconset
sips -z 16 16     app-icon.png --out AppIcon.iconset/icon_16x16.png
sips -z 32 32     app-icon.png --out AppIcon.iconset/icon_16x16@2x.png
sips -z 32 32     app-icon.png --out AppIcon.iconset/icon_32x32.png
sips -z 64 64     app-icon.png --out AppIcon.iconset/icon_32x32@2x.png
sips -z 128 128   app-icon.png --out AppIcon.iconset/icon_128x128.png
sips -z 256 256   app-icon.png --out AppIcon.iconset/icon_128x128@2x.png
sips -z 256 256   app-icon.png --out AppIcon.iconset/icon_256x256.png
sips -z 512 512   app-icon.png --out AppIcon.iconset/icon_256x256@2x.png
sips -z 512 512   app-icon.png --out AppIcon.iconset/icon_512x512.png
sips -z 1024 1024 app-icon.png --out AppIcon.iconset/icon_512x512@2x.png
iconutil -c icns AppIcon.iconset -o assets/icons/app-icon.icns
```

## Best Practices

1. **Don't commit large files**: Keep icon files reasonably sized (< 1MB each)
2. **Use PNG for transparency**: Always use PNG format for icons that need transparent backgrounds
3. **Maintain aspect ratio**: Keep icons square (1:1 aspect ratio)
4. **Test across platforms**: Verify icons display correctly on macOS, Windows, Linux
5. **Version control**: Track changes to assets in git commits

## Updating Icons

When updating application icons:

1. Generate new icon using Ideogram or design tool
2. Convert to PNG with transparent background (use ImageMagick or remove.bg)
3. Resize to 512x512: `sips -z 512 512 new-icon.png --out assets/icons/app-icon.png`
4. Create retina version: `sips -z 1024 1024 assets/icons/app-icon.png --out assets/icons/app-icon@2x.png`
5. Test build: `cargo build --release -p nameback-gui`
6. Commit changes with descriptive message: `git acm "feat: update app icon to new design"`

## References

- [Apple Human Interface Guidelines - App Icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)
- [ImageMagick Documentation](https://imagemagick.org/script/command-line-options.php)
- [create-dmg Documentation](https://github.com/create-dmg/create-dmg)
