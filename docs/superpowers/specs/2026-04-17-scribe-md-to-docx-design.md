# Folio — Markdown → DOCX Converter (Design Spec)

**Date:** 2026-04-17
**Status:** Draft v1
**Author:** brainstormed with Livia_Tassel
**Target release:** macOS (.dmg, universal binary) + Windows (.exe/.msi, x64 + arm64)

---

## 1. Product Summary

**Folio** is a desktop application that converts Markdown (`.md`) files into Microsoft Word documents (`.docx`) **without requiring manual cleanup** of formatting afterward.

The core promise: *paste or drop a Markdown file, get a .docx that opens in Word looking correct on the first try.*

Where existing tools (Pandoc, Typora export, Marked 2, mdbook) require after-the-fact fixes — especially for math formulas, image sizing, table layout, and Chinese academic conventions — Folio produces output that is ready to submit, print, or share.

---

## 2. Target Users

| User | Primary Use | Priority Features |
|---|---|---|
| **Individual technical writers (self)** | Daily Markdown → Word handoff | Drag-drop, sensible defaults, live preview |
| **Scholars (EN)** | Papers, theses, reports | LaTeX math → native OMML, figure/equation numbering, cross-refs, IEEE/ACM templates |
| **Corporate tech writers** | Internal docs, proposals | Ingest corporate `reference.docx`, batch conversion, style mapping, tables |
| **Chinese academic users** | 毕业论文, 期刊投稿, 学报 | 宋体/黑体 defaults, 三线表, GB/T-style equation numbering, 中文 captions (图/表), full-width punctuation, 毕业论文 templates |

The Chinese-academic slice is where competitors are weakest — the key differentiation wedge.

---

## 3. Goals & Non-Goals

### 3.1 Goals (v1)

- Produce `.docx` output that requires **zero manual formatting cleanup** for the supported feature set.
- Render **LaTeX math as native Word equations (OMML)**, fully editable inside Word, not as images.
- Handle images with automatic sizing, aspect-ratio preservation, captions, and cross-references.
- Ship **bundled templates** covering the most common use cases (≥5).
- Accept **user-supplied `reference.docx`** to extract and apply corporate/personal style sets.
- Provide a **live side-by-side preview** that matches final `.docx` output closely (≥95% visual fidelity for supported features).
- Support **batch conversion** of folders.
- Ship as **signed, notarized builds** on macOS (DMG) and Windows (MSI), usable on a fresh machine without warnings.

### 3.2 Non-Goals (v1)

- DOCX → Markdown (reverse conversion) — v2.
- Collaborative editing / cloud sync.
- Bibliography / citation management (BibTeX, GB/T 7714) — v1.1.
- Mobile / web versions.
- Real-time multi-user editing.
- Converting to formats other than `.docx` (PDF, HTML, slides) — out of scope.

### 3.3 Explicit feature scope for v1

**Markdown features supported:**
- CommonMark + GFM: headings (H1–H6), paragraphs, bold/italic/strikethrough, inline code, lists (ordered/unordered, nested), blockquotes, fenced code blocks with language tags, tables, horizontal rules, links, task lists, footnotes.
- Extended: LaTeX math (`$...$` inline, `$$...$$` block, `\begin{align}...\end{align}`), image sizing syntax `![alt|width=60%](img.png)`, caption syntax `![caption text](img.png "Caption")`, table alignment, auto-numbered figures/tables/equations with cross-references (`[@fig:label]`, `[@eq:label]`, `[@tbl:label]`).
- HTML passthrough: limited — `<br>`, `<sup>`, `<sub>`, and a configurable allowlist.

**Output fidelity guarantees:**
- All headings map to template's `Heading 1…N` styles.
- All math renders as editable OMML (equation numbering right-aligned with parens for Chinese style, or inline with label-ref for Western).
- All images fit page width by default, respect explicit sizing, include captions numbered per chapter.
- All tables fit page width, support 三线表 preset.
- All code blocks use monospace style with optional syntax highlighting baked as character runs.
- Footnotes render as native Word footnotes.
- Auto-generated TOC if `[[TOC]]` marker present.

---

## 4. Architecture Overview

### 4.1 High-Level Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  Tauri 2 Desktop App (signed .dmg + .msi)                   │
│                                                             │
│  ┌──────────────────────┐    ┌──────────────────────────┐   │
│  │ Frontend (Svelte)    │◄──►│ Rust Backend (Tauri cmd) │   │
│  │                      │    │                          │   │
│  │  • File picker /     │    │  • MD parser             │   │
│  │    drag-drop         │    │    (pulldown-cmark)      │   │
│  │  • Editor (optional) │    │  • LaTeX → MathML        │   │
│  │  • Preview pane      │    │    (latex2mathml)        │   │
│  │  • Template picker   │    │  • MathML → OMML         │   │
│  │  • Settings          │    │    (custom transformer)  │   │
│  │  • Batch queue UI    │    │  • Image processor       │   │
│  │                      │    │    (image crate)         │   │
│  └──────────────────────┘    │  • Syntax highlighter    │   │
│                              │    (syntect)             │   │
│                              │  • Template engine       │   │
│                              │    (reads reference.docx)│   │
│                              │  • DOCX writer           │   │
│                              │    (docx-rs, bokuweb)    │   │
│                              │  • Preview renderer      │   │
│                              │    (docx → HTML approx.) │   │
│                              └──────────────────────────┘   │
│                                                             │
│  Single binary per platform, no runtime deps, ~15 MB.       │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Core Pipeline

```
 .md file/string
     ↓
[1] Read + normalize (UTF-8, BOM strip, line endings)
     ↓
[2] pulldown-cmark parse → event stream
     ↓
[3] AST builder (our own typed AST, easier to transform)
     ↓
[4] Pre-processors:
     • math extraction (LaTeX → MathML tree nodes)
     • image resolution (path → bytes + dimensions + captions)
     • cross-ref resolution (figures, tables, equations)
     • syntax highlight code blocks (syntect)
     • auto-number figures/tables/equations per template rules
     ↓
[5] DOCX emitter:
     • template context (styles from active .docx template)
     • walk AST → emit paragraphs, runs, tables, images, OMML
     • assemble document.xml + styles.xml + numbering.xml + media/
     ↓
[6] docx-rs serializes → .docx zip
     ↓
[7] (Preview branch) AST → HTML approximation for live pane
```

### 4.3 Module Boundaries

Each module is independently testable, communicates via plain typed values, and has a single clear responsibility.

**Rust crates (workspace):**

| Crate | Purpose | Depends on |
|---|---|---|
| `scribe-ast` | Our typed MD AST (nodes for headings, paragraphs, math, images, tables, code, cross-refs) | — |
| `scribe-parser` | Markdown → AST. Wraps pulldown-cmark, handles extensions (math, image-sizing syntax, cross-ref labels) | pulldown-cmark, scribe-ast |
| `scribe-math` | LaTeX → MathML → OMML. Pure Rust. | latex2mathml, scribe-ast, quick-xml |
| `scribe-images` | Image loading, sizing normalization, format conversion, dimension extraction | image crate, scribe-ast |
| `scribe-highlight` | Syntax highlighting for code blocks → styled character runs | syntect, scribe-ast |
| `scribe-template` | Load `reference.docx`, extract style map (heading styles, body style, code style, table style, font defaults), load bundled templates | docx-rs, zip, quick-xml |
| `scribe-docx` | Emit `.docx` from AST given a template. The heart. | docx-rs, scribe-ast, scribe-math, scribe-images, scribe-highlight, scribe-template |
| `scribe-preview` | AST → HTML approximation for live preview pane | scribe-ast |
| `scribe-core` | Orchestration: file I/O, pipeline, batch runner | all of the above |
| `scribe-tauri` | Tauri 2 app shell: commands, IPC, filesystem permissions | tauri, scribe-core |

**Frontend (Svelte + TypeScript):**
- `src/routes/+page.svelte` — main UI (editor + preview)
- `src/lib/editor/` — optional CodeMirror editor
- `src/lib/preview/` — HTML preview pane
- `src/lib/templates/` — template picker
- `src/lib/batch/` — batch queue UI
- `src/lib/settings/` — settings panel
- `src/lib/ipc.ts` — typed wrapper around Tauri `invoke`

### 4.4 Data Flow for Single-File Conversion

1. User drops `paper.md` onto the window.
2. Frontend sends `{ input_path, template_id, output_path }` via Tauri `invoke("convert_file", …)`.
3. `scribe-tauri::commands::convert_file` calls `scribe-core::convert_file`.
4. `scribe-core` reads file, calls `scribe-parser::parse` → AST.
5. Pre-processors run (math, images, cross-refs, highlighting).
6. `scribe-template::load_template(template_id_or_path)` → `StyleMap`.
7. `scribe-docx::emit(ast, style_map)` → `Vec<u8>` (.docx bytes).
8. Bytes written to `output_path`.
9. Frontend shows success, offers "Reveal in Finder / Explorer".

### 4.5 Data Flow for Live Preview

1. Editor fires a debounced change event (500 ms).
2. Frontend `invoke("render_preview", { markdown, template_id })`.
3. Backend runs parse + preprocessors (no image bytes, no DOCX emit).
4. `scribe-preview::render(ast, style_map)` → HTML string.
5. Frontend injects HTML into preview iframe with bundled CSS matching template.

Preview is an *approximation*, not byte-exact to DOCX, but matches: fonts, colors, heading sizes, math rendering (via MathJax-like inline SVG from our MathML), image positioning, table style, code block colors, footnote anchors.

---

## 5. Technology Stack

### 5.1 Desktop Shell

**Tauri 2.x**
- Rust backend + webview frontend.
- Cross-platform: macOS (universal arm64 + x86_64), Windows (x64 + arm64).
- Small bundle (~15 MB vs Electron's ~150 MB).
- Native look on each OS.
- Signed + notarized builds via Tauri's bundler.
- **Avoid sidecars / `externalBin`** on macOS — known notarization issues. All dependencies must be pure Rust.

### 5.2 Frontend

**Svelte 5 + SvelteKit (SPA mode) + TypeScript + Vite**
- Small bundle, reactive, minimal boilerplate.
- **CodeMirror 6** for optional editor pane (syntax highlighting, virtual scroll for large files).
- **Tailwind CSS v4** for styling, with a design token system matching native OS accent color.

### 5.3 Markdown Parsing

**pulldown-cmark 0.12+**
- Pull parser, fast, SIMD-accelerated.
- Built-in support: CommonMark, GFM tables, footnotes, strikethrough, task lists, math.
- We extend with: image-sizing syntax, cross-ref labels, caption syntax, TOC marker.

### 5.4 Math: LaTeX → OMML

**Pipeline: LaTeX → MathML → OMML (all pure Rust)**

1. `latex2mathml` crate: LaTeX → MathML (Presentation MathML).
2. Custom `scribe-math` module: MathML → OMML. Implemented by porting Microsoft's `mml2omml.xsl` XSLT 1.0 stylesheet to a direct Rust AST transformer. ~1,500–2,500 LOC.

**Rationale:**
- Shelling out to `pandoc` (which uses `texmath`) works but adds a runtime dependency and hits Tauri sidecar notarization bugs on macOS.
- Embedding an XSLT engine in Rust is heavier than porting the transformer directly.
- The MathML → OMML mapping is well-documented and stable (OOXML spec has not changed).
- Pure Rust = single binary, fast, no fork/exec, works on any arch.

**Supported math features for v1:**
- Inline `$...$` and block `$$...$$`.
- Common constructs: `\frac`, `\sqrt`, `\sum`, `\int`, `\prod`, `\lim`, matrices (matrix/pmatrix/bmatrix/vmatrix), `\begin{align}`, `\begin{cases}`, Greek letters, decorations (`\hat`, `\bar`, `\vec`, `\dot`), subscripts/superscripts, spacing, standard operators.
- Equation numbering: auto-incremented per document, right-aligned with parentheses `(1)` by default. Per-template configurable (chapter-prefixed `(1.1)` for Chinese theses).
- `\label{eq:foo}` + `\ref{eq:foo}` or `[@eq:foo]` for cross-references.

**Explicitly not in v1:**
- TikZ, chemfig, mhchem, custom `\newcommand` definitions, complex package-specific macros.

### 5.5 DOCX Writing

**docx-rs (bokuweb) 0.4.19+**
- Mature (1M+ downloads, active 2026 releases).
- Builder API for paragraphs, runs, tables, images, footnotes, numbering, styles.
- **Gap we fill ourselves:** OMML math emission. docx-rs does not natively expose OMML run construction — we emit OMML XML strings and insert as raw paragraph content via the library's extensibility hooks (contributing a math run type back upstream if possible).

### 5.6 Images

**`image` crate (^0.25)** + **`imageops`**
- Supports PNG, JPEG, GIF, WebP, BMP, TIFF.
- Extract dimensions for auto-sizing.
- Normalize SVG to PNG for embedding (via `resvg` crate) since Word's SVG support is fragile.
- Auto-resize to page width (minus margins from active template) if image exceeds it, preserving aspect ratio.

### 5.7 Syntax Highlighting

**`syntect`** with bundled themes.
- Tokenize code blocks per language.
- Emit styled character runs (not raw HTML).
- Default theme: "InspiredGitHub" (light) with template-configurable alternatives.

### 5.8 Template System

**Format:** Standard `.docx` files, bundled in `templates/`.
**Mechanism:**
- On load, we unzip the template, parse `styles.xml`, extract: heading styles (names, fonts, sizes, colors, spacing), body style, code style, table styles, list styles, page margins, default fonts, equation-numbering style.
- We then construct a `StyleMap` that the DOCX emitter uses to map AST nodes → correct style IDs.
- User-supplied `reference.docx`: same loading path; we inspect styles and map by name convention (e.g., style named "Heading 1" → H1).

**Bundled templates (v1):**

| ID | Name | Use |
|---|---|---|
| `academic-en` | Academic (English) | General English papers, Times New Roman 12pt, 1.5 spacing |
| `ieee` | IEEE Conference | Two-column IEEE style (approximation, single-column fallback) |
| `corporate-minimal` | Corporate Minimal | Clean Arial/Calibri, corporate-report friendly |
| `thesis-cn-ugrad` | 中文本科毕业论文 | 宋体正文 12pt, 黑体标题, GB/T equation numbering (chapter-prefixed), 三线表, A4, 2.54cm margins |
| `journal-cn` | 中文期刊通用 | 宋体/黑体, 图/表 captions, 摘要/关键词 style anchors |

### 5.9 Preview Rendering

Pure HTML + CSS + inline SVG for math (we emit MathML, browsers render it natively; fall back to SVG via `mathml-core` CSS for consistent cross-browser look).
Per-template CSS file approximates page margins, fonts, heading sizes, table styles.

### 5.10 Configuration & Persistence

- `~/Library/Application Support/Folio/` (macOS) / `%APPDATA%\Folio\` (Windows)
- `settings.json`: last-used template, window size, UI preferences.
- `templates/`: user-imported `reference.docx` files.
- `recent.json`: recent file list.
- No telemetry in v1 (opt-in crash reporting in v1.1).

### 5.11 Testing

- **Unit tests** per crate (`cargo test`).
- **Integration tests**: fixture `.md` files → `.docx`, validated by parsing output XML and asserting invariants (headings have correct style IDs, math is OMML, images have correct dimensions).
- **Golden tests**: known-good `.docx` outputs for key fixtures; diff XML after normalization.
- **Visual regression**: open generated `.docx` in headless LibreOffice → render PNG → perceptual diff (optional, in CI).
- **Cross-platform CI**: GitHub Actions matrix — macOS 14 + Windows 2022.

### 5.12 Build, Sign, Ship

**macOS:**
- Build: `cargo tauri build --target universal-apple-darwin`.
- Sign with **Developer ID Application** cert (Apple Developer account required — $99/yr).
- Notarize via App Store Connect API key (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`).
- Output: signed `.dmg`.
- Hardened runtime enabled, entitlements: `com.apple.security.cs.allow-jit` (WebView requirement).

**Windows:**
- Build: `cargo tauri build --target x86_64-pc-windows-msvc` (and `aarch64-pc-windows-msvc`).
- Sign with **EV code-signing certificate** via Azure Key Vault (preferred) or imported PFX.
- Output: signed `.msi` (WiX) and `.exe` (NSIS).
- SmartScreen reputation accrues fastest with EV certs.

**Distribution:**
- v1: direct download from a simple landing page + Gumroad (handles payment, licensing).
- v1.1: Microsoft Store + Mac App Store submission (App Sandbox requirements to be validated).
- Chinese market: Alipay / WeChat Pay via Paddle for Pro tier.

---

## 6. Monetization

| Tier | Price | Features |
|---|---|---|
| **Free** | $0 | Single-file conversion, 2 bundled templates (`academic-en`, `corporate-minimal`), live preview |
| **Pro** | $29 one-time, or $5/mo | All bundled templates (incl. Chinese academic), batch conversion, custom `reference.docx` ingestion, priority updates, 1 year of updates on one-time purchase |
| **Team** (v1.1+) | $12/mo per seat, min 3 seats | Shared template library, company-wide style sync |

License enforcement: offline license keys signed with ed25519, verified on launch. No phone-home requirement.

---

## 7. Release Plan

### 7.1 v1.0 milestones

**M1 — Walking skeleton (Week 1)**
- Tauri 2 scaffolding, Rust workspace, Svelte frontend builds, IPC "hello" works.
- pulldown-cmark → trivial DOCX (just paragraphs) proves end-to-end pipeline.

**M2 — Core conversion (Week 2)**
- Full AST, headings, paragraphs, lists, blockquotes, inline formatting, code blocks + syntect, tables, footnotes, links.
- `academic-en` template working with a bundled `reference.docx`.
- CLI harness `scribe-cli` for fixture testing.

**M3 — Math + images + cross-refs (Week 3)**
- `scribe-math` implemented: LaTeX → MathML → OMML.
- Image embedding with sizing, captions, cross-references.
- Equation numbering (simple + chapter-prefixed).
- Figure/table numbering.

**M4 — Templates + preview + batch (Week 4)**
- All 5 bundled templates.
- `reference.docx` ingestion.
- Live preview pane.
- Batch mode UI + backend.

**M5 — Polish + signing + release (Week 5)**
- Settings UI, recent files, drag-drop polish.
- macOS signing + notarization, Windows signing.
- Landing page, Gumroad integration.
- Beta release to 10–20 testers.

**M6 — v1.0 public release (Week 6)**
- Bug fixes from beta.
- App Store / Microsoft Store submission (post-launch).

### 7.2 v1.1 (target: +6 weeks after v1.0)

- Citations: BibTeX + GB/T 7714.
- More templates: top Chinese journals, arXiv preprint, dissertation variants (硕士/博士).
- Corporate style-mapping UI.
- Team tier.

### 7.3 v2 (target: +3 months)

- DOCX → Markdown (round-trip).
- Export to PDF / HTML / slides.
- Plugin system for custom filters.

---

## 8. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| MathML → OMML transformer has edge cases | High | Medium | Large fixture set covering real-world papers; fallback to render-math-as-image with warning; accept contributions |
| Windows EV cert acquisition lead time | Medium | Medium | Start cert application early in week 1; ship unsigned beta to early testers meanwhile |
| Apple notarization failures on universal builds | Medium | Medium | Use official Tauri 2 path; avoid sidecars entirely; test on fresh macOS VMs |
| Chinese font rendering requires embedded fonts | Medium | Low | Use system 宋体/黑体 (universal on Win/Mac with Chinese locale); document that English-only machines need to install fonts to see output correctly (Word handles this) |
| Preview diverges noticeably from DOCX | High | Low | Document limitations; "Open in Word" button for exact check |
| docx-rs missing OMML support | High | High | Accept upstream PR or vendor the minimal fork; already planned as custom work |
| User's Markdown uses unsupported LaTeX macros | Medium | Medium | Fail gracefully: emit the raw LaTeX as code with warning; show a list of unsupported constructs in UI |
| Scope creep (users demand more formats) | High | Medium | v1 is one-way MD→DOCX, period. Route other formats to v2. |

---

## 9. Success Criteria

v1.0 is successful if:

1. A 20-page English academic paper with 30+ equations, 15+ figures, 5+ tables converts cleanly with zero manual fixes in Word.
2. A 50-page Chinese undergraduate thesis (毕业论文) converts with correct 宋体/黑体, 三线表, chapter-prefixed equation numbering, and Chinese captions.
3. Batch conversion of 20 `.md` files completes in under 30 seconds.
4. Live preview updates within 1 second of typing.
5. Both `.dmg` (universal) and `.msi` (x64) install and launch on a fresh machine without Gatekeeper / SmartScreen warnings.
6. ≥50 paying users in first 60 days post-launch.

---

## 10. Open Questions

These are resolved before the plan is finalized, or at the start of each milestone:

1. **IEEE template in v1 or v1.1?** True two-column IEEE is non-trivial; a single-column approximation is acceptable for v1.
2. **Font embedding in output DOCX?** Embedding fonts bloats files but guarantees rendering on any machine. Default: do not embed; make it a template setting.
3. **SVG image handling: rasterize or pass through?** Word's SVG support is inconsistent across versions. Default: rasterize to PNG via resvg for v1.
4. **License verification: fully offline or occasional online check?** Offline for v1.0 (simpler, better UX); revisit if piracy becomes an issue.
5. **Should the editor pane be required, or optional?** Proposal: the default UI is "drop a file, see preview, export." The editor is a toggle, not mandatory.

---

## 11. Out of Scope (explicit)

- Real-time collaborative editing.
- Cloud storage / sync.
- Mobile apps (iOS, Android).
- Web version (browser-based converter).
- Conversion *from* formats other than Markdown.
- Conversion *to* formats other than DOCX.
- Custom macro / plugin system (deferred to v2+).
- OCR of images.
- Translation of text content.
- Grammar / spell checking.

---

## Appendix A: Example Converted Fragments

**Input (MD):**
```
## Results

The energy is given by $E = mc^2$. For the general case:

$$E^2 = (mc^2)^2 + (pc)^2 \quad \text{(1)}$$

See Figure [@fig:setup] for the experimental setup.

![Experimental setup|width=80%](setup.png "Experimental setup")
```

**Expected output in Word:**
- "Results" as Heading 2 with template's H2 style.
- Inline `E = mc²` as editable OMML equation.
- Block equation, centered, with auto-number `(1)` right-aligned per template.
- "See Figure 1 for the experimental setup." with "Figure 1" as a live cross-reference field.
- Image at 80% of text width, centered, aspect-ratio preserved, caption "Figure 1: Experimental setup" below.

---

## Appendix B: Chinese Academic Fixture (abridged)

**Input:**
```
## 实验结果

能量由 $E = mc^2$ 给出。对于一般情况：

$$E^2 = (mc^2)^2 + (pc)^2 \tag{2.1}$$

如表 [@tbl:results] 所示。

| 样本 | 能量 (J) | 误差 |
|------|---------|------|
| A    | 12.3    | 0.1  |
| B    | 45.6    | 0.2  |

: 实验测量结果 {#tbl:results}
```

**Expected output:**
- "实验结果" as Heading 2 (黑体 四号 / 14pt bold).
- 宋体 body text, 小四号 / 12pt.
- Inline equation editable.
- Block equation with right-aligned `(2.1)` (chapter-prefixed per thesis template).
- Table rendered as 三线表 (top thick line, header-body thin line, bottom thick line only), caption "表 2-1  实验测量结果" above table with live cross-reference.
