# Folio

Folio is a cross-platform desktop application and Rust workspace for turning Markdown into Microsoft Word documents with native Word structures instead of brittle export artifacts. The focus is practical fidelity: editable equations, sane image sizing, readable tables, predictable heading hierarchy, and output that looks intentional when it opens in Word or LibreOffice.

<img src="docs/images/folio-logo.png" alt="Folio logo" width="160" />

![Folio conversion pipeline](docs/images/pipeline.svg)

## Why This Exists

Most Markdown-to-DOCX workflows break down in the last 10%:

- math is flattened into images or mangled XML
- lists and tables need manual repair
- images overflow, shrink unpredictably, or lose aspect ratio
- academic and report-style formatting still needs hand cleanup in Word

Folio is built to close that gap with a pure Rust pipeline and a desktop UX around it.

## Demo

The sample below is generated from the comprehensive regression fixture in [`test/folio-comprehensive.md`](test/folio-comprehensive.md), rendered from the produced [`test/output/folio-comprehensive.docx`](test/output/folio-comprehensive.docx).

### Sample Export

![Folio sample export](docs/images/demo-sample-export.png)

What the current pipeline already demonstrates:

- visible heading hierarchy for `H1` to `H6`
- unordered, ordered, and task lists with preserved start values
- inline and display LaTeX math rendered as editable OMML
- syntax-highlighted code blocks
- native footnotes
- GFM tables with alignment
- raster and SVG image embedding

## Current Scope

Folio is still **pre-alpha**, but the core conversion path is already structured as a serious multi-crate system rather than a prototype script.

Implemented today:

- Markdown parsing with CommonMark/GFM features
- typed AST for downstream transforms
- LaTeX -> MathML -> OMML conversion
- image loading, normalization, and SVG rasterization
- DOCX emission with styles, numbering, footnotes, tables, and images
- HTML preview rendering for the desktop app
- Tauri desktop shell with a Svelte frontend

Planned / not complete yet:

- reference-template ingestion from user `reference.docx`
- richer academic presets and style packs
- cross-references and auto-numbered figures/tables/equations
- batch conversion UX polish
- higher-fidelity preview styling parity

## Technology Stack

### Core conversion engine

- Rust stable workspace
- `pulldown-cmark` for Markdown parsing
- `latex2mathml` plus a custom `MathML -> OMML` transformer
- `docx-rs` for OpenXML document generation
- `image` + `resvg` for raster and SVG asset handling
- `syntect` for syntax highlighting
- `zip` + `quick-xml` for DOCX package post-processing

### Desktop app

- Tauri 2 for the native desktop shell
- Svelte 5 + SvelteKit for the frontend
- Vite for frontend build tooling
- Tailwind CSS 4 for styling
- TypeScript for the UI layer

### Output model

- native Word paragraphs, runs, numbering, and footnotes
- editable OMML equations instead of equation screenshots
- embedded raster media for predictable image portability
- HTML preview generated from the same AST used for DOCX emission

## Workspace Layout

The product is now branded as **Folio**. Internal crate and package names still use the historical `scribe-*` prefix for now so the workspace can evolve without a large package rename.

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
fixtures/           Sample Markdown inputs used for testing and demos
docs/               Design docs, plans, and README assets
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

## Fixture-Based Testing

The repository includes fixture documents under [`fixtures/`](fixtures/) and a comprehensive all-in-one regression fixture under [`test/`](test/).

Useful local commands:

```bash
cargo run -p scribe-cli -- fixtures/english/m2-kitchen-sink.md -o /tmp/scribe-m2.docx
cargo run -p scribe-cli -- fixtures/english/m3-math.md -o /tmp/scribe-m3.docx
cargo run -p scribe-cli -- test/folio-comprehensive.md -o test/output/folio-comprehensive.docx
```

If you have LibreOffice installed, rendering the generated `.docx` files to PDF/PNG is a practical way to inspect layout regressions before shipping changes.

## Design Notes

Longer-form planning and design material lives here:

- [`docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md`](docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md)
- [`docs/superpowers/plans/2026-04-17-scribe-v1-plan.md`](docs/superpowers/plans/2026-04-17-scribe-v1-plan.md)

## License

MIT. See [`LICENSE`](LICENSE).
