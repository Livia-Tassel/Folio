<p align="center">
  <img src="docs/images/folio-logo-mark.png" alt="Folio logo" width="220" />
</p>

<h1 align="center">Folio</h1>

<p align="center">
  Markdown to polished <code>.docx</code> output, without the cleanup pass.
</p>

<p align="center">
  <a href="README.md"><strong>English</strong></a>
  ·
  <a href="README-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/Livia-Tassel/Folio/stargazers">
    <img src="https://badgen.net/github/stars/Livia-Tassel/Folio?icon=github&label=stars" alt="GitHub stars" />
  </a>
  <a href="https://github.com/Livia-Tassel/Folio/network/members">
    <img src="https://badgen.net/github/forks/Livia-Tassel/Folio?icon=github&label=forks" alt="GitHub forks" />
  </a>
  <a href="https://github.com/Livia-Tassel/Folio/issues">
    <img src="https://badgen.net/github/open-issues/Livia-Tassel/Folio?icon=github&label=issues" alt="GitHub issues" />
  </a>
  <a href="https://github.com/Livia-Tassel/Folio/commits/master">
    <img src="https://img.shields.io/github/last-commit/Livia-Tassel/Folio?style=flat-square" alt="Last commit" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/github/license/Livia-Tassel/Folio?style=flat-square" alt="License" />
  </a>
</p>

<p align="center">
  <img src="docs/images/pipeline.svg" alt="Folio conversion pipeline" width="100%" />
</p>

Folio is a cross-platform desktop application and Rust workspace for turning Markdown into Microsoft Word documents with native Word structures instead of brittle export artifacts. The focus is practical fidelity: editable equations, sane image sizing, readable tables, predictable heading hierarchy, and output that looks intentional when it opens in Word or LibreOffice.

## Quick Start

### For most users

The simplest way to use Folio on both macOS and Windows is to download a prebuilt installer from [GitHub Releases](https://github.com/Livia-Tassel/Folio/releases).

At the moment, the published desktop releases are still **experimental** and may not work reliably on every machine. If you need the most dependable path today, use the CLI from source instead.

- macOS: download the `.dmg` that matches your CPU.
- Apple Silicon: choose the `aarch64` / `arm64` build.
- Intel Mac: choose the `x64` build.
- Open the `.dmg`, then drag **Folio** into **Applications**.
- Windows: download the NSIS `.exe` installer for the simplest setup.

Because Folio is still pre-alpha and may be unsigned, the first launch may show a platform warning:

- macOS: right-click **Folio** in Applications and choose **Open** the first time if Gatekeeper blocks it.
- Windows: if SmartScreen appears, click **More info** -> **Run anyway**.

### If you want to build from source

Use the development setup below only if you want to contribute or run the app from source.

### Recommended CLI Usage

If the desktop release does not work for you yet, the CLI is the recommended fallback.

macOS:

```bash
cd /path/to/Folio
cargo run -p scribe-cli -- "/absolute/path/input.md" -o "/absolute/path/output.docx"
```

Windows (PowerShell):

```powershell
cd C:\path\to\Folio
cargo run -p scribe-cli -- "C:\absolute\path\input.md" -o "C:\absolute\path\output.docx"
```

You can also omit `-o`, and Folio will write `<input>.docx` next to the source Markdown file.

## Why Folio

Most Markdown-to-DOCX workflows break down in the last 10%:

- math is flattened into images or mangled XML
- lists and tables need manual repair
- images overflow, shrink unpredictably, or lose aspect ratio
- report and academic formatting still needs hand cleanup in Word

Folio is built to close that gap with a pure Rust pipeline and a desktop UX around it.

## Sample Export

The samples below are generated from the comprehensive regression fixture in [`test/folio-comprehensive.md`](test/folio-comprehensive.md), exported to [`test/output/folio-comprehensive.docx`](test/output/folio-comprehensive.docx), and rendered from the corresponding PDF.

<table>
  <tr>
    <td width="33.33%" valign="top">
      <img src="test/output/sample-o.png" alt="Folio sample page 1" width="100%" />
    </td>
    <td width="33.33%" valign="top">
      <img src="test/output/sample-r.png" alt="Folio sample page 2" width="100%" />
    </td>
    <td width="33.33%" valign="top">
      <img src="test/output/sample-t.png" alt="Folio sample page 3" width="100%" />
    </td>
  </tr>
  <tr>
    <td align="center"><strong>Page 1</strong><br/>Formatting, lists, code, and tables</td>
    <td align="center"><strong>Page 2</strong><br/>Math and raster logo embedding</td>
    <td align="center"><strong>Page 3</strong><br/>SVG assets and footnotes</td>
  </tr>
</table>

<p align="center">
  <a href="test/output/folio-comprehensive.docx">Download DOCX</a>
  ·
  <a href="test/output/folio-comprehensive.pdf">Open PDF</a>
  ·
  <a href="test/folio-comprehensive.md">View Source Markdown</a>
</p>

The current sample exercises:

- heading hierarchy
- inline emphasis, code, and links
- unordered, ordered, and task lists
- blockquotes and code blocks
- aligned tables
- inline and display LaTeX math rendered as editable OMML
- raster and SVG image embedding
- footnotes

## WPS Spot Check vs Pandoc

The comparison below uses the same source note rendered as `.docx`, then opened in **WPS Office** for a practical end-user check.

<table>
  <tr>
    <td width="33.33%" valign="top">
      <img src="test/output/Folio.png" alt="Folio sample page 1" width="100%" />
    </td>
    <td width="33.33%" valign="top">
      <img src="test/output/Pandoc.png" alt="Folio sample page 2" width="100%" />
    </td>
    </td>
  </tr>
  <tr>
    <td align="center"><strong>Folio</strong></td>
    <td align="center"><strong>Pandoc</strong></td>
  </tr>
</table>

In this note, Folio is materially stronger than Pandoc in the places that matter most for technical writing:

- formula-heavy sections stay closer to the intended reading order instead of collapsing into inline-like fragments
- image + heading + paragraph blocks hold together more predictably across pages
- long structured notes stay denser and cleaner instead of ballooning into many extra pages

## What Works Today

Folio is still **pre-alpha**, but the core conversion path is already structured as a serious multi-crate system rather than a prototype script.

Implemented today:

- Markdown parsing with CommonMark and GFM features
- typed AST for downstream transforms
- LaTeX -> MathML -> OMML conversion
- image loading, normalization, and SVG rasterization
- DOCX emission with styles, numbering, footnotes, tables, and images
- HTML preview rendering for the desktop app
- Tauri desktop shell with a Svelte frontend

Not complete yet:

- reference-template ingestion from user `reference.docx`
- richer academic presets and style packs
- cross-references and auto-numbered figures, tables, and equations
- batch conversion UX polish
- higher-fidelity preview parity

## Technology Stack

### Core conversion engine

- Rust stable workspace
- `pulldown-cmark` for Markdown parsing
- `latex2mathml` plus a custom `MathML -> OMML` transformer
- `docx-rs` for OpenXML document generation
- `image` and `resvg` for raster and SVG asset handling
- `syntect` for syntax highlighting
- `zip` and `quick-xml` for DOCX package post-processing

### Desktop app

- Tauri 2 for the native desktop shell
- Svelte 5 + SvelteKit for the frontend
- Vite for frontend build tooling
- Tailwind CSS 4 for styling
- TypeScript for the UI layer

### Output model

- native Word paragraphs, runs, numbering, and footnotes
- editable OMML equations instead of screenshots
- embedded raster media for portable image rendering
- HTML preview generated from the same AST used for DOCX emission

## Repository Layout

The product is branded as **Folio**. Internal crate and package names still use the historical `scribe-*` prefix for now so the workspace can evolve without a broad package rename.

```text
crates/
  scribe-ast        Typed Markdown AST
  scribe-parser     Markdown -> AST
  scribe-math       LaTeX -> MathML -> OMML
  scribe-images     Image loading and sizing
  scribe-highlight  Code highlighting
  scribe-template   Template/style plumbing
  scribe-docx       AST -> .docx emission
  scribe-preview    AST -> HTML preview
  scribe-core       Shared orchestration API
  scribe-tauri      Desktop shell and frontend bridge
scribe-cli/         CLI wrapper for fixture testing
fixtures/           Focused sample inputs
test/               Comprehensive regression fixture and export artifacts
docs/               Design docs and README assets
```

## Development

### Requirements

- Rust stable
- Node.js 20+
- `pnpm`

### Install frontend dependencies

```bash
pnpm --dir crates/scribe-tauri/frontend install
```

### Run tests

```bash
cargo test --workspace
pnpm --dir crates/scribe-tauri/frontend check
```

### Start the desktop app

```bash
cd crates/scribe-tauri
cargo tauri dev
```

### Build the frontend only

```bash
pnpm --dir crates/scribe-tauri/frontend build
```

## Shipping Releases

For end users, GitHub Releases should be the default distribution channel.

1. Bump the workspace version in [`Cargo.toml`](Cargo.toml).
2. Create and push a tag such as `v0.1.2`.
3. GitHub Actions will build and publish:
   - macOS Apple Silicon `.dmg`
   - macOS Intel `.dmg`
   - Windows NSIS `.exe`

The release workflow also generates platform icon assets from [`crates/scribe-tauri/icons/icon.png`](crates/scribe-tauri/icons/icon.png) before bundling.

## Fixture-Based Testing

The repository includes focused fixtures under [`fixtures/`](fixtures/) and a comprehensive all-in-one regression fixture under [`test/`](test/).

Useful local commands:

```bash
cargo run -p scribe-cli -- fixtures/english/m2-kitchen-sink.md -o /tmp/folio-m2.docx
cargo run -p scribe-cli -- fixtures/english/m3-math.md -o /tmp/folio-m3.docx
cargo run -p scribe-cli -- test/folio-comprehensive.md -o test/output/folio-comprehensive.docx
soffice --headless --convert-to pdf --outdir test/output test/output/folio-comprehensive.docx
```

If you have LibreOffice installed, rendering generated `.docx` files to PDF and PNG is a practical way to inspect layout regressions before shipping changes.

## Design Notes

Longer-form planning and design material lives here:

- [`docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md`](docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md)
- [`docs/superpowers/plans/2026-04-17-scribe-v1-plan.md`](docs/superpowers/plans/2026-04-17-scribe-v1-plan.md)

## GitHub Activity

<p align="center">
  <a href="https://github.com/Livia-Tassel/Folio/stargazers">
    <img src="https://badgen.net/github/stars/Livia-Tassel/Folio?icon=github&label=stars" alt="Folio GitHub stars" />
  </a>
  <a href="https://github.com/Livia-Tassel/Folio/network/members">
    <img src="https://badgen.net/github/forks/Livia-Tassel/Folio?icon=github&label=forks" alt="Folio GitHub forks" />
  </a>
  <a href="https://github.com/Livia-Tassel/Folio/issues">
    <img src="https://badgen.net/github/open-issues/Livia-Tassel/Folio?icon=github&label=issues" alt="Folio GitHub issues" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/Livia-Tassel/Folio">
    <img src="https://github-readme-stats.vercel.app/api/pin/?username=Livia-Tassel&repo=Folio&theme=transparent&show_owner=true" alt="Folio repository card" />
  </a>
</p>

## License

MIT. See [`LICENSE`](LICENSE).
