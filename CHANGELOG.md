# Changelog

All notable changes to Folio are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The Rust workspace, the desktop app, the CLI, and the `folio-docx`
Python wheels all share one version.

## [Unreleased]

## [0.1.4] — 2026-05-01

### Added
- **Python bindings** (`folio-docx` on PyPI). A new `crates/scribe-py`
  PyO3 wrapper exposes `convert`, `convert_file`, `preview_html`, and
  `preview_standalone` to Python. Wheels are abi3 (one wheel per
  platform covers CPython 3.8+) and ship for macOS (Apple Silicon +
  Intel), Linux x86_64 + aarch64 (manylinux 2_28), and Windows x64.
  Conversion releases the GIL, so multi-threaded callers don't
  serialise on Folio.
- `scripts/py-dev.sh` — idempotent local setup. Creates `.venv`,
  installs maturin + pytest, runs `maturin develop --release`, and
  works around a macOS `.pth`-quarantine quirk that silently breaks
  `import folio` on freshly-created venvs.
- GitHub Actions `python.yml` workflow — builds the full wheel matrix
  on every PR + tag, installs each wheel on its native runner and
  runs the pytest smoke suite against it, and on tag push publishes
  to PyPI via OIDC trusted publishing (no API token in repo secrets).

### Fixed
- **East-Asian font fallback in code styles.** Code blocks and inline
  code previously declared only ASCII / hi-ANSI fonts (Menlo /
  Consolas), so a CJK character inside `code` rendered in whatever
  fallback Word picked — usually not monospaced. Pin
  `w:eastAsia="PingFang SC"` on the SourceCode + InlineCode styles
  via a single `code_fonts()` helper. Word still substitutes if the
  named face is missing on the host.

## [0.1.3] — 2026-04-29

### Fixed
- Preview and DOCX regressions surfaced after the v0.1.2 release.

## [0.1.2] — 2026-04-26

### Added
- WPS comparison documentation against Pandoc.

### Fixed
- Windows release pipeline: NSIS now publishes only on Windows runners.
- Windows icon resource is now embedded in the app binary.
- Tauri build commands resolve project paths correctly when invoked
  from the workspace root.

## [0.1.1] — 2026-04-23

### Added
- Bilingual README (English + 简体中文).

### Changed
- Cropped logo, GitHub badge layout, and README-CN translation polish.

## [0.1.0] — 2026-04-22

Initial public release. Markdown → `.docx` with native Word
structures: editable equations (LaTeX → OMML), syntax-highlighted
code blocks, footnotes, page-fit images, GFM tables. Cross-platform
Tauri 2 desktop shell, SvelteKit live-preview pane, `scribe-cli` for
headless conversion, CI matrix on macOS, Windows, and Ubuntu.

[Unreleased]: https://github.com/Livia-Tassel/Folio/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/Livia-Tassel/Folio/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/Livia-Tassel/Folio/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Livia-Tassel/Folio/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Livia-Tassel/Folio/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Livia-Tassel/Folio/releases/tag/v0.1.0
