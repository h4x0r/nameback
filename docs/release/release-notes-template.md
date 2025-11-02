# Release Notes Template

Use this template when creating GitHub releases. Keep it simple and user-focused.

## Template

```markdown
## What's New
- [List 2-4 key user-facing changes]
- [Focus on benefits, not technical details]

## How to Install

### Windows
Download `nameback-x86_64-pc-windows-msvc.msi` below and run it. Everything installs automatically.

### macOS
**GUI users (recommended):** Download the DMG installer below for your chip:
- Apple Silicon (M1/M2/M3): `nameback-aarch64-apple-darwin.dmg`
- Intel: `nameback-x86_64-apple-darwin.dmg`

**Command line:**
```bash
brew install --cask nameback
```

### Linux (Ubuntu/Debian/Kali)
```bash
wget https://github.com/h4x0r/nameback/releases/download/v{VERSION}/nameback_{VERSION}-1_amd64.deb
sudo dpkg -i nameback_{VERSION}-1_amd64.deb
sudo apt-get install -f
```

### Linux (Other)
```bash
cargo install nameback
```

That's it! Run `nameback` (command line) or `nameback-gui` (visual app).
```

## Example (v0.6.18)

```markdown
## What's New
- Better GUI colors for light mode (easier to read)
- Professional macOS installer with Security Ronin branding
- Smarter window resizing in GUI

## How to Install

### Windows
Download `nameback-x86_64-pc-windows-msvc.msi` below and run it. Everything installs automatically.

### macOS
**GUI users (recommended):** Download the DMG installer below for your chip:
- Apple Silicon (M1/M2/M3): `nameback-aarch64-apple-darwin.dmg`
- Intel: `nameback-x86_64-apple-darwin.dmg`

**Command line:**
```bash
brew install --cask nameback
```

### Linux (Ubuntu/Debian/Kali)
```bash
wget https://github.com/h4x0r/nameback/releases/download/v0.6.18/nameback_0.6.18-1_amd64.deb
sudo dpkg -i nameback_0.6.18-1_amd64.deb
sudo apt-get install -f
```

### Linux (Other)
```bash
cargo install nameback
```

That's it! Run `nameback` (command line) or `nameback-gui` (visual app).
```

## Guidelines

**DO:**
- Focus on user benefits, not implementation details
- Use simple, direct language
- Show exactly what command to run
- Keep it short (users skim, not read)

**DON'T:**
- Explain technical architecture
- Add comparison tables
- Include security verification steps (advanced users know how)
- Provide multiple installation options for same platform
- Use marketing language or emojis

**Remember:** Users just want to know:
1. What changed?
2. How do I get it?

That's it.
