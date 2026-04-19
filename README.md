# Scribe

> Markdown → Word (.docx) conversion that you don't have to fix afterward.

Cross-platform desktop app (macOS + Windows). Pure Rust backend, Svelte frontend, Tauri 2 shell.

**Status:** pre-alpha, actively implementing v1.

## What it solves

Existing converters (Pandoc, Typora export, etc.) produce `.docx` that opens in Word but needs manual cleanup — especially for:

- LaTeX math formulas (often rendered as images instead of editable equations)
- Image sizing, captioning, and cross-references
- Table overflow
- Chinese academic style (宋体/黑体, 三线表, equation numbering with parens)

Scribe emits native Word-style `.docx` — math as editable OMML, images sized to page, figures/tables/equations auto-numbered and cross-referenced, bundled templates for common academic and corporate styles.

## Development

Requires: Rust stable, pnpm, Node 20+.

```bash
cargo check --workspace
cd crates/scribe-tauri && cargo tauri dev
```

See [`docs/superpowers/specs/`](docs/superpowers/specs/) and [`docs/superpowers/plans/`](docs/superpowers/plans/) for the full design and implementation plan.

## License

MIT. See [LICENSE](LICENSE).
