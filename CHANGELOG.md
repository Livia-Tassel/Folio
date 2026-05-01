# Changelog

All notable changes to Folio are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The Rust workspace, the desktop app, the CLI, and the `folio-docx`
Python wheels all share one version.

## [Unreleased]

## [0.2.0] — 2026-05-01

The "branded Word out, no cleanup pass" release. Folio now ships a
real **template system** that turns the README's tagline from
aspiration into a one-flag operation.

### Added

- **Template system.** A new `scribe-template` crate loads the
  styles from a Word reference document and the rest of the pipeline
  honours them on emit. Three knobs at the user-facing layer:
  - `scribe-cli paper.md --theme <name>` — pick a built-in theme.
  - `scribe-cli paper.md --reference-doc <path>` — supply your own
    styled `.docx`.
  - `scribe-cli --list-themes` — enumerate built-in themes.
  These are mutually exclusive and surface clear errors when both
  are given.
- **Page-setup inheritance from reference docs.** When you supply
  a `--reference-doc`, Folio now also lifts the body-terminating
  `<w:sectPr>` out of the reference's `word/document.xml` and
  splices it into the output. Page margins, paper size, columns,
  headers/footers anchors all flow through with the styles, so a
  reference designed for A5 with custom margins actually produces
  A5 output with custom margins. Built-in themes do not carry a
  sectPr — they only override styles, leaving Folio's default page
  setup intact.
- **Built-in themes.** Three themes ship with 0.2.0, baked into the
  binary at compile time so every install carries them:
  - `academic` — Times New Roman 12pt body, 1.5 line height,
    classic English academic-paper hierarchy.
  - `thesis-cn` — 宋体 正文 + 黑体 标题 + 1.5 倍行距 + 首行缩进
    2 字符. Targets the largest defensible default for Chinese
    学位论文.
  - `report` — Calibri sans-serif, blue heading accents, tighter
    line spacing. For status updates, one-pagers, internal memos.
- **Python parity.** The same controls reach Python:
  - `folio.convert(md, theme="academic")`
  - `folio.convert(md, reference_doc="my-template.docx")`
  - `folio.list_themes()`
  Updated type stubs, docstrings, and 7 new pytest cases covering
  theme + reference-doc end-to-end.
- **`scribe_template::Template`** public API:
  `from_reference_doc(path)`, `from_reference_doc_bytes(bytes)`,
  `from_styles_xml(xml)`, `builtin(name)`, plus
  `list_builtin_themes()` for discovery. `section_xml()` exposes
  the lifted page setup for callers that want it.
- **`scribe_docx::EmitOptions`** master entry. The previous
  `emit` and `emit_with_base` are now thin wrappers; new callers
  use `emit_with_options(doc, opts)` and threaded a
  `styles_xml_override` and `section_xml_override` through to
  generic `postprocess_styles` / `postprocess_section` zip passes
  that mirror the existing math postprocess pattern.
- **`scribe_core::convert_*_with_template`** entry points; new
  `ConvertError::Template` variant routes loader errors with
  helpful context.

### Changed

- README and README-CN gained a "Templates" section right after
  the Python quick-start, with a side-by-side theme table and
  CLI/Python examples.

### Tests

- Rust workspace: 102 tests, all green.
- Python: 16 tests, all green (8 from 0.1.4 + 8 new for templates +
  page setup).
- Mutation-tested critical match arms in `scribe-template` to
  confirm tests guard the intended branches.
- CI workflow `python.yml` (added in 0.1.4) builds wheels on
  Linux x86_64 + aarch64, macOS x86_64 + aarch64, and Windows x64,
  installs each wheel on a native runner, and runs the pytest
  smoke suite — so 0.2.0 ships with the real cross-platform
  guarantee, not just a green local build.

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

[Unreleased]: https://github.com/Livia-Tassel/Folio/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Livia-Tassel/Folio/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/Livia-Tassel/Folio/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/Livia-Tassel/Folio/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Livia-Tassel/Folio/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Livia-Tassel/Folio/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Livia-Tassel/Folio/releases/tag/v0.1.0
