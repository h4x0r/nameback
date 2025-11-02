# Nameback Documentation

Welcome to the Nameback documentation. This directory contains comprehensive documentation for developers, contributors, and maintainers.

## Quick Links

- **[User Guide](../README.md)** - Main project README with installation and usage instructions
- **[AI Assistant Guide](../CLAUDE.md)** - Instructions for Claude Code when working on this project
- **[Release Process](../RELEASING.md)** - How to create and publish releases

## Documentation Structure

### 📋 User Documentation
End-user guides and tutorials.

- [`../README.md`](../README.md) - Main project documentation
- **[User Guide](user/guide.md)** - Comprehensive user guide

### 🛠️ Development
Development guides, coding standards, and implementation details.

- **[TDD Implementation](dev/tdd-implementation.md)** - Test-Driven Development summary and test coverage
- **[Refactoring Plan](dev/refactoring-plan.md)** - Current refactoring work (Phase 1 complete)
- **[Refactoring Opportunities](dev/refactoring-opportunities.md)** - Code quality analysis and improvement opportunities
- **[Kali Linux Packaging](dev/kali-packaging.md)** - Debian packaging for Kali Linux

### 🏗️ Architecture
Technical design documents and implementation analyses.

- **[Naming Heuristics](architecture/naming-heuristics.md)** - Algorithm for extracting meaningful file names
- **[Bundled Dependencies: Legal Analysis](architecture/bundled-dependencies-legal-analysis.md)** - License compliance for bundling dependencies
- **[Bundled Dependencies: Implementation](architecture/bundled-dependencies-implementation.md)** - 4-layer fallback system implementation details

### 🚀 Release
Release management, templates, and processes.

- **[Release Process](../RELEASING.md)** - Automated release workflow (keep in root)
- **[Release Notes Template](release/release-notes-template.md)** - Template for writing release notes

### 🔒 Security
Security policies and vulnerability reporting.

- **[Security Policy](security/)** - Vulnerability reporting and security practices

---

## Document Categories

### Root-Level Documentation (Keep Minimal)
These files stay in the project root for discoverability:
- `README.md` - Main project documentation
- `CLAUDE.md` - AI assistant instructions
- `RELEASING.md` - Quick reference for releases
- `LICENSE` - Project license
- `CHANGELOG.md` - Version history (auto-generated)

### Development Documentation (`docs/dev/`)
Internal development guides, coding practices, refactoring plans:
- TDD implementation notes
- Refactoring plans and opportunities
- Coding standards and conventions
- Development workflow guides

### Architecture Documentation (`docs/architecture/`)
Technical design decisions, implementation analyses:
- System architecture
- Dependency management
- Implementation summaries
- Technical decision records (ADRs)

### Release Documentation (`docs/release/`)
Release-related templates and guides:
- Release note templates
- Version upgrade guides
- Breaking change migration guides

### User Documentation (`docs/user/`)
End-user guides and tutorials (future):
- Installation guides
- Usage tutorials
- Feature walkthroughs
- FAQ

---

## Contributing to Documentation

When adding new documentation:

1. **Choose the right category**
   - User-facing? → `docs/user/`
   - Development process? → `docs/dev/`
   - Technical design? → `docs/architecture/`
   - Release-related? → `docs/release/`

2. **Use descriptive filenames**
   - Good: `TDD_IMPLEMENTATION.md`, `BUNDLED_DEPENDENCIES.md`
   - Avoid: `notes.md`, `temp.md`, `doc1.md`

3. **Update this index**
   - Add your new document to the appropriate section above

4. **Link from root README**
   - If the documentation is important for new users/contributors

---

## Document History

This documentation structure was established in **November 2025** to organize the growing collection of development documents.

**Previous State:** 9 markdown files scattered in root directory
**Current State:** Organized into `docs/` with clear categorization
**Next Steps:** Continue populating user guides and architecture docs
