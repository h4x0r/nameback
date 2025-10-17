# Security Controls Architecture

## 🎯 What's Installed in Your Repository

This document explains the security architecture deployed in your project by 1-Click GitHub Security.

### 📊 Performance & Coverage Metrics
| Metric | Value | Impact |
|--------|--------|--------|
| **Pre-Push Validation** | < 60 seconds | ⚡ Developer workflow preservation |
| **Security Controls** | 35+ comprehensive | 🛡️ Complete attack vector coverage |
| **Language Support** | Multi-language | 🌐 Universal project compatibility |
| **Issue Resolution Speed** | 10x faster | 🚀 Early detection advantage |

## 🏗️ Two-Tier Security Architecture

### Tier 1: Pre-Push Controls (< 60 seconds)
**Purpose**: Block critical issues before they enter the repository

**Controls Installed**:
- ✅ **Secret Detection** - Blocks API keys, passwords, tokens (gitleakslite)
- ✅ **Vulnerability Scanning** - Catches known security issues (language-specific)
- ✅ **Code Quality** - Linting and formatting validation (language-specific)
- ✅ **Test Validation** - Ensures tests pass before push (language-specific)
- ✅ **Supply Chain Security** - SHA pinning, dependency validation (pinactlite)
- ✅ **License Compliance** - Validates dependency licenses (language-specific)

### Tier 2: Post-Push Controls (CI/CD Analysis)
**Purpose**: Comprehensive analysis and reporting

**Workflows Installed** (optional, via --workflows flag):
- 🔍 **Static Analysis** - SAST with CodeQL and Trivy
- 🔍 **Dependency Auditing** - Automated vulnerability detection
- 🔍 **Security Reporting** - SBOM generation and metrics
- 🔍 **Compliance Checking** - License and policy validation

## 🔧 Components Installed

### Pre-Push Hook
**Location**: `.git/hooks/pre-push`
**Function**: Runs security validation before every push
**Performance**: Completes in < 60 seconds
**Bypass**: `git push --no-verify` (emergency use only)

### Security Tools
**Location**: `.security-controls/bin/`
- `gitleakslite` - Secret detection (embedded binary)
- `pinactlite` - GitHub Actions SHA pinning (embedded binary)

### Configuration Files
- `.security-controls-version` - Tracks installed version
- `.security-controls-config` - Installation configuration
- Language-specific configs (e.g., `.cargo/audit.toml`, `.eslintrc.js`)

### Optional CI/CD Workflows
**Location**: `.github/workflows/`
- `security-ci-workflow.yml` - Comprehensive security analysis
- Additional specialized workflows (if --workflows used)

## 🚀 Developer Workflow Integration

### Normal Development
1. **Code** - Write code as usual
2. **Commit** - `git commit` works normally
3. **Push** - Pre-push hook validates automatically (< 60s)
4. **CI** - Optional comprehensive analysis runs in background

### When Pre-Push Fails
The hook provides specific fix instructions:

```bash
# Format issues
cargo fmt --all                    # Rust
npm run format                     # Node.js
black .                           # Python
go fmt ./...                      # Go

# Linting issues
cargo clippy --all-targets --fix  # Rust
npm run lint --fix               # Node.js
flake8 . --fix                   # Python
golint ./...                     # Go

# Security vulnerabilities
cargo audit fix                   # Rust
npm audit fix                    # Node.js
safety check                     # Python
govulncheck ./...               # Go

# Secrets detected
# Remove secrets, use environment variables

# GitHub Actions not SHA-pinned
.security-controls/bin/pinactlite pinactlite --dir .github/workflows
```

## 🔐 GitHub Security Features (Optional)

When installed with `--github-security` flag:

### Automatically Configured
- **Dependabot Vulnerability Alerts** - Automated dependency scanning
- **Dependabot Security Fixes** - Automated security update PRs
- **Branch Protection Rules** - Requires reviews and status checks
- **CodeQL Security Scanning** - Automated code analysis
- **Secret Scanning** - Server-side secret detection
- **Secret Push Protection** - Blocks secrets at GitHub level

## 🎯 Language-Specific Security

### Rust Projects
- `cargo audit` - Vulnerability scanning
- `cargo clippy` - Security linting
- `cargo test` - Test validation
- `cargo license` - License compliance

### Node.js Projects
- `npm audit` - Vulnerability scanning
- `eslint` - Security linting
- `npm test` - Test validation
- `license-checker` - License compliance

### Python Projects
- `safety check` - Vulnerability scanning
- `bandit` - Security linting
- `pytest` - Test validation
- `pip-licenses` - License compliance

### Go Projects
- `govulncheck` - Vulnerability scanning
- `golint` - Security linting
- `go test` - Test validation
- `go-licenses` - License compliance

### Generic Projects
- Universal secret detection
- GitHub Actions SHA pinning
- Basic file validation

## 🛠️ Maintenance and Updates

### Upgrading Security Controls
```bash
# Download latest installer
curl -O https://github.com/h4x0r/1-click-github-sec/releases/latest/download/install-security-controls.sh

# Run upgrade (preserves your settings)
chmod +x install-security-controls.sh
./install-security-controls.sh --upgrade
```

### Manual Tool Updates
```bash
# Update embedded security tools
.security-controls/bin/gitleakslite --update
.security-controls/bin/pinactlite --update
```

### Configuration Management
- Version tracked in `.security-controls-version`
- Settings preserved in `.security-controls-config`
- Backups created in `.security-controls-backup/`

## 📊 Monitoring and Metrics

### Pre-Push Performance
- Target: < 60 seconds total execution time
- Parallel execution for efficiency
- Tool-specific timeouts prevent hangs

### Security Coverage
- 35+ comprehensive security controls
- Multi-language vulnerability detection
- Supply chain attack prevention
- Secret exposure prevention

### Compliance Standards
- ✅ NIST SSDF aligned
- ✅ SLSA Level 2 compliant
- ✅ OpenSSF best practices
- ✅ SBOM generation ready

## 🔗 Additional Resources

- **Complete Architecture**: https://h4x0r.github.io/1-click-github-sec/architecture
- **Installation Guide**: https://h4x0r.github.io/1-click-github-sec/installation
- **Cryptographic Verification**: https://h4x0r.github.io/1-click-github-sec/cryptographic-verification
- **GitHub Repository**: https://github.com/h4x0r/1-click-github-sec

---

*This architecture document describes the security controls installed in your specific repository. For complete technical details, see the full documentation at the links above.*
