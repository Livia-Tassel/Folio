# Folio v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship Folio v1 — a signed, cross-platform (macOS .dmg + Windows .msi) desktop app that converts Markdown to `.docx` with zero post-conversion cleanup, including LaTeX math as native OMML, image sizing, cross-references, and Chinese academic templates.

**Architecture:** Tauri 2 shell (Rust backend + Svelte frontend). Rust workspace with 9 focused crates handling parsing, math, images, syntax highlighting, templates, DOCX emission, preview, orchestration, and the Tauri shell. Pipeline: MD → AST → pre-processors → DOCX. Pure Rust end-to-end (no sidecars) to avoid macOS notarization issues.

**Tech Stack:** Tauri 2, Rust (stable), Svelte 5 + SvelteKit SPA + TypeScript + Vite, Tailwind v4, CodeMirror 6, pulldown-cmark, latex2mathml, docx-rs (bokuweb), image + resvg, syntect, quick-xml, zip. CI: GitHub Actions macOS 14 + Windows 2022.

**Spec:** `docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md`

---

## Plan Structure

- **§A. File & Module Map** — full project layout, locked before coding starts.
- **§B. Milestone M1 — Walking skeleton** (detailed TDD tasks)
- **§C. Milestone M2 — Core conversion** (detailed TDD tasks)
- **§D. Milestone M3 — Math, images, cross-refs** (task-level outline; expand at M3 kickoff)
- **§E. Milestone M4 — Templates, preview, batch** (outline)
- **§F. Milestone M5 — Polish, signing, release** (outline)
- **§G. Milestone M6 — Public v1.0** (outline)
- **§H. Cross-cutting concerns** (CI, versioning, release process)

Each milestone ends with a commit-and-tag step and a verification checklist.

---

## §A. File & Module Map

### Rust workspace layout

```
scribe/
├── Cargo.toml                        # workspace definition
├── rust-toolchain.toml               # pin stable
├── .cargo/config.toml                # build flags
├── crates/
│   ├── scribe-ast/                   # typed MD AST
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                # enums: Node, Inline, Block, MathKind, etc.
│   ├── scribe-parser/                # MD → AST
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # parse() entry
│   │       ├── events.rs             # pulldown-cmark → AST builder
│   │       ├── extensions.rs         # image-size syntax, cross-ref labels
│   │       └── tests/fixtures/       # .md fixture files
│   ├── scribe-math/                  # LaTeX → MathML → OMML
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── latex.rs              # wraps latex2mathml
│   │       ├── mml_to_omml.rs        # the ported transformer
│   │       └── tests/fixtures/       # .tex → expected OMML XML
│   ├── scribe-images/                # image loading + resizing
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── scribe-highlight/             # syntect wrapper → styled runs
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── scribe-template/              # reference.docx ingestion + bundled templates
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs                # StyleMap
│   │   │   ├── loader.rs             # unzip + parse styles.xml
│   │   │   └── bundled.rs            # include_bytes! of bundled .docx
│   │   └── bundled/                  # the 5 .docx templates
│   │       ├── academic-en.docx
│   │       ├── ieee.docx
│   │       ├── corporate-minimal.docx
│   │       ├── thesis-cn-ugrad.docx
│   │       └── journal-cn.docx
│   ├── scribe-docx/                  # AST → .docx bytes
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # emit() entry
│   │       ├── paragraphs.rs
│   │       ├── tables.rs
│   │       ├── images.rs
│   │       ├── math.rs               # insert OMML math runs
│   │       ├── footnotes.rs
│   │       ├── numbering.rs          # figure/table/equation counters
│   │       └── crossref.rs
│   ├── scribe-preview/               # AST → HTML approx
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── scribe-core/                  # orchestration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # convert_file, convert_string, batch
│   │       └── error.rs              # unified error type
│   └── scribe-tauri/                 # Tauri 2 app shell
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── build.rs
│       ├── icons/
│       └── src/
│           ├── main.rs
│           └── commands.rs           # #[tauri::command] fns
└── scribe-cli/                       # small CLI harness for fixture testing
    ├── Cargo.toml
    └── src/main.rs
```

### Frontend layout (inside `crates/scribe-tauri/`)

```
scribe-tauri/
├── frontend/
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.ts
│   ├── index.html
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css
│   │   ├── routes/
│   │   │   ├── +layout.svelte
│   │   │   └── +page.svelte
│   │   ├── lib/
│   │   │   ├── ipc.ts                # typed invoke wrapper
│   │   │   ├── stores/               # svelte stores for state
│   │   │   │   ├── document.ts
│   │   │   │   ├── template.ts
│   │   │   │   └── settings.ts
│   │   │   ├── editor/
│   │   │   │   └── Editor.svelte
│   │   │   ├── preview/
│   │   │   │   └── Preview.svelte
│   │   │   ├── templates/
│   │   │   │   └── TemplatePicker.svelte
│   │   │   ├── batch/
│   │   │   │   └── BatchQueue.svelte
│   │   │   └── settings/
│   │   │       └── SettingsPanel.svelte
│   │   └── components/
│   │       ├── TitleBar.svelte
│   │       ├── DropZone.svelte
│   │       └── Toast.svelte
│   └── static/
```

### Repo-level files

```
scribe/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    # test + build matrix
│   │   └── release.yml               # signed builds + release artifacts
│   └── ISSUE_TEMPLATE/
├── docs/
│   ├── superpowers/                  # specs + plans (this file's parent)
│   ├── README.md
│   └── CONTRIBUTING.md
├── templates-src/                    # source files used to produce bundled .docx
│   └── README.md                     # how to regenerate bundled templates
├── fixtures/                         # integration fixtures (.md + expected .docx)
│   ├── english/
│   └── chinese/
├── .gitignore
├── LICENSE
└── README.md
```

---

## §B. Milestone M1 — Walking Skeleton (Week 1)

**Goal:** Tauri 2 app launches on macOS and Windows, with a Rust workspace compiling, frontend rendering, IPC proven, and a trivial MD → DOCX round-trip (paragraphs only) working end-to-end.

**Exit criteria:**
- `cargo tauri dev` launches a window on both macOS and Windows.
- Dropping `hello.md` (containing `# Hello\nWorld`) produces a `.docx` with H1 "Hello" and paragraph "World".
- CI pipeline runs `cargo test` + `pnpm build` on both platforms.
- Signed+notarized macOS debug build and signed Windows debug build work on a clean VM.

### Task M1-1: Initialize Rust workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `rust-toolchain.toml`
- Create: `.cargo/config.toml`
- Create: `.gitignore`
- Create: `README.md`
- Create: `LICENSE` (MIT or Apache-2.0 dual — decide here; default MIT for broadest use)

- [ ] **Step 1: Create workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
    "crates/scribe-ast",
    "crates/scribe-parser",
    "crates/scribe-math",
    "crates/scribe-images",
    "crates/scribe-highlight",
    "crates/scribe-template",
    "crates/scribe-docx",
    "crates/scribe-preview",
    "crates/scribe-core",
    "crates/scribe-tauri",
    "scribe-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/YOUR_ORG/scribe"

[workspace.dependencies]
anyhow = "1"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
quick-xml = "0.36"
zip = "2"
pulldown-cmark = { version = "0.12", default-features = false, features = ["simd"] }
latex2mathml = "0.2"
image = { version = "0.25", default-features = false, features = ["png","jpeg","gif","bmp","webp","tiff"] }
resvg = "0.44"
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
docx-rs = "0.4"
tauri = { version = "2", features = [] }
tauri-build = "2"
```

- [ ] **Step 2: Create `rust-toolchain.toml`**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 3: Create `.gitignore`**

```gitignore
/target
/crates/scribe-tauri/frontend/node_modules
/crates/scribe-tauri/frontend/.svelte-kit
/crates/scribe-tauri/frontend/build
/crates/scribe-tauri/dist
/fixtures/**/*.docx
.DS_Store
.env
.env.*
!.env.example
*.log
```

- [ ] **Step 4: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: "warning: virtual manifest has no members that exist" until crates are added; that's acceptable. When crate stubs land in next tasks, `cargo check` passes.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml .gitignore .cargo README.md LICENSE
git commit -m "chore: initialize Rust workspace"
```

### Task M1-2: Stub all crates with passing `cargo check`

For each of the 10 crates listed in §A, create `Cargo.toml` + `src/lib.rs` (or `src/main.rs` for scribe-cli and scribe-tauri) with a trivial compiling stub.

**Files:**
- Create: `crates/scribe-ast/Cargo.toml`, `crates/scribe-ast/src/lib.rs`
- …repeat for each crate…

- [ ] **Step 1: Write stub files for all 10 crates**

Each `Cargo.toml` follows this minimal template:

```toml
[package]
name = "scribe-ast"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
```

Each `src/lib.rs` is:

```rust
//! scribe-ast: typed Markdown AST for Folio.

#[cfg(test)]
mod tests {
    #[test]
    fn it_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
```

- [ ] **Step 2: Verify `cargo check --workspace` and `cargo test --workspace` pass**

Run: `cargo check --workspace && cargo test --workspace`
Expected: all 10 crates compile, 10 `it_compiles` tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/ scribe-cli/
git commit -m "chore: stub all workspace crates"
```

### Task M1-3: Scaffold SvelteKit frontend

**Files:**
- Create: `crates/scribe-tauri/frontend/` (via `pnpm create svelte`)

- [ ] **Step 1: Scaffold with SvelteKit**

Run (one-off, interactive):
```bash
cd crates/scribe-tauri
pnpm create svelte@latest frontend \
  --template skeleton \
  --types typescript \
  --no-prettier --no-eslint --no-playwright --no-vitest
cd frontend
pnpm install
pnpm add -D @tauri-apps/cli@next @tauri-apps/api@next
pnpm add -D tailwindcss@next @tailwindcss/vite@next
pnpm add codemirror @codemirror/lang-markdown @codemirror/state @codemirror/view
```

- [ ] **Step 2: Configure SvelteKit for SPA mode**

Edit `svelte.config.js` to use `adapter-static` with `fallback: "index.html"`. Edit `vite.config.ts` to add the tailwind plugin. Replace `+layout.svelte` with `<script>export const prerender = true; export const ssr = false;</script> <slot />`.

- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: `build/` dir with `index.html`, no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/scribe-tauri/frontend
git commit -m "chore: scaffold SvelteKit SPA frontend"
```

### Task M1-4: Initialize Tauri 2 shell

**Files:**
- Create: `crates/scribe-tauri/tauri.conf.json`
- Create: `crates/scribe-tauri/build.rs`
- Create: `crates/scribe-tauri/src/main.rs`
- Create: `crates/scribe-tauri/icons/` (generate via `tauri icon`)
- Modify: `crates/scribe-tauri/Cargo.toml`

- [ ] **Step 1: Add Tauri deps to `scribe-tauri/Cargo.toml`**

```toml
[package]
name = "scribe-tauri"
version.workspace = true
edition.workspace = true
license.workspace = true

[build-dependencies]
tauri-build = { workspace = true }

[dependencies]
tauri = { workspace = true, features = ["macos-private-api"] }
serde = { workspace = true }
serde_json = { workspace = true }
scribe-core = { path = "../scribe-core" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 2: Create `tauri.conf.json`**

```json
{
  "$schema": "../../node_modules/@tauri-apps/cli/schema.json",
  "productName": "Folio",
  "version": "0.1.0",
  "identifier": "com.scribe.app",
  "build": {
    "beforeDevCommand": "pnpm --dir frontend dev",
    "beforeBuildCommand": "pnpm --dir frontend build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "./frontend/build"
  },
  "app": {
    "windows": [
      {
        "title": "Folio",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi", "nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "hardenedRuntime": true,
      "minimumSystemVersion": "11.0"
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

- [ ] **Step 3: Create `build.rs` and `src/main.rs`**

`build.rs`:
```rust
fn main() { tauri_build::build() }
```

`src/main.rs`:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn ping() -> &'static str { "pong" }
```

- [ ] **Step 4: Generate icons**

Prepare a 1024×1024 placeholder PNG (a stylized "S" on gradient — use any tool, commit the source). Run:
```bash
cd crates/scribe-tauri
pnpm tauri icon ./icon-source.png
```

- [ ] **Step 5: Wire frontend to call `ping`**

In `frontend/src/routes/+page.svelte`:
```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  let message = $state("");
  async function call() {
    message = await invoke<string>("ping");
  }
</script>
<button onclick={call}>Ping backend</button>
<p>{message}</p>
```

- [ ] **Step 6: Launch dev and verify IPC works**

Run: `cargo tauri dev` (from `crates/scribe-tauri/`)
Expected: window opens, clicking button shows "pong".

- [ ] **Step 7: Commit**

```bash
git add crates/scribe-tauri
git commit -m "feat(shell): tauri 2 skeleton with working ipc ping"
```

### Task M1-5: Implement trivial MD → DOCX (paragraphs + H1 only)

This proves the end-to-end pipeline with the smallest possible surface.

**Files:**
- Modify: `crates/scribe-ast/src/lib.rs`
- Modify: `crates/scribe-parser/src/lib.rs`, add dep on `scribe-ast` + `pulldown-cmark`
- Modify: `crates/scribe-docx/src/lib.rs`, add dep on `scribe-ast` + `docx-rs`
- Modify: `crates/scribe-core/src/lib.rs`, add deps on parser + docx
- Modify: `crates/scribe-tauri/src/main.rs` + new `src/commands.rs`
- Test: `crates/scribe-core/tests/smoke.rs`

- [ ] **Step 1: Write the failing integration test**

`crates/scribe-core/tests/smoke.rs`:
```rust
use std::io::Read;

#[test]
fn converts_hello_world_md_to_docx() {
    let md = "# Hello\n\nWorld";
    let bytes = scribe_core::convert_string(md).expect("convert");

    // DOCX is a zip — quick sanity: starts with "PK"
    assert_eq!(&bytes[0..2], b"PK", "not a zip");

    // Unzip, pull document.xml, assert it mentions Hello and World
    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();

    assert!(xml.contains("Hello"));
    assert!(xml.contains("World"));
}
```

- [ ] **Step 2: Run, verify it fails**

Run: `cargo test -p scribe-core smoke`
Expected: compile error — `convert_string` undefined.

- [ ] **Step 3: Define minimal AST in `scribe-ast`**

```rust
//! scribe-ast: typed Markdown AST.

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph { text: String },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document { pub blocks: Vec<Block> }
```

- [ ] **Step 4: Implement trivial parser**

`scribe-parser/src/lib.rs`:
```rust
use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel};
use scribe_ast::{Block, Document};

pub fn parse(md: &str) -> Document {
    let mut doc = Document::default();
    let mut current_heading: Option<u8> = None;
    let mut buffer = String::new();
    let mut in_paragraph = false;

    for event in Parser::new(md) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some(match level {
                    HeadingLevel::H1 => 1, HeadingLevel::H2 => 2, HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4, HeadingLevel::H5 => 5, HeadingLevel::H6 => 6,
                });
                buffer.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let lvl = current_heading.take().unwrap();
                doc.blocks.push(Block::Heading { level: lvl, text: std::mem::take(&mut buffer) });
            }
            Event::Start(Tag::Paragraph) => { in_paragraph = true; buffer.clear(); }
            Event::End(TagEnd::Paragraph) => {
                in_paragraph = false;
                doc.blocks.push(Block::Paragraph { text: std::mem::take(&mut buffer) });
            }
            Event::Text(t) => { buffer.push_str(&t); }
            _ => {}
        }
    }
    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_h1_and_paragraph() {
        let d = parse("# Hello\n\nWorld");
        assert_eq!(d.blocks, vec![
            Block::Heading { level: 1, text: "Hello".into() },
            Block::Paragraph { text: "World".into() },
        ]);
    }
}
```

Add to `scribe-parser/Cargo.toml`:
```toml
[dependencies]
scribe-ast = { path = "../scribe-ast" }
pulldown-cmark = { workspace = true }
```

- [ ] **Step 5: Implement trivial docx emitter**

`scribe-docx/src/lib.rs`:
```rust
use docx_rs::{Docx, Paragraph, Run};
use scribe_ast::{Block, Document};

pub fn emit(doc: &Document) -> anyhow::Result<Vec<u8>> {
    let mut out = Docx::new();
    for block in &doc.blocks {
        let p = match block {
            Block::Heading { level, text } => {
                Paragraph::new()
                    .style(&format!("Heading{}", level))
                    .add_run(Run::new().add_text(text))
            }
            Block::Paragraph { text } => {
                Paragraph::new().add_run(Run::new().add_text(text))
            }
        };
        out = out.add_paragraph(p);
    }
    let mut buf = Vec::new();
    out.build().pack(&mut buf)?;
    Ok(buf)
}
```

Add deps to `scribe-docx/Cargo.toml`:
```toml
[dependencies]
scribe-ast = { path = "../scribe-ast" }
docx-rs = { workspace = true }
anyhow = { workspace = true }
```

- [ ] **Step 6: Wire `scribe-core::convert_string`**

`scribe-core/src/lib.rs`:
```rust
pub fn convert_string(md: &str) -> anyhow::Result<Vec<u8>> {
    let doc = scribe_parser::parse(md);
    scribe_docx::emit(&doc)
}
```

Add deps:
```toml
[dependencies]
scribe-parser = { path = "../scribe-parser" }
scribe-docx  = { path = "../scribe-docx" }
anyhow = { workspace = true }

[dev-dependencies]
zip = { workspace = true }
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test -p scribe-core smoke -- --nocapture`
Expected: PASS.

- [ ] **Step 8: Add Tauri command and UI**

`crates/scribe-tauri/src/commands.rs`:
```rust
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn convert_file(input_path: String, output_path: String) -> Result<(), String> {
    let md = fs::read_to_string(&input_path).map_err(|e| e.to_string())?;
    let bytes = scribe_core::convert_string(&md).map_err(|e| e.to_string())?;
    fs::write(&output_path, bytes).map_err(|e| e.to_string())?;
    Ok(())
}
```

Update `main.rs` to include it in the handler.

Frontend: drop-zone component that takes a file, calls `convert_file` with `input_path` and `input_path.replace(".md", ".docx")`.

- [ ] **Step 9: Manual smoke test**

Run: `cargo tauri dev`, drag `hello.md` containing `# Hello\nWorld` onto window.
Expected: `hello.docx` appears next to `hello.md`, opens in Word/Pages showing H1 "Hello" and paragraph "World".

- [ ] **Step 10: Commit**

```bash
git add .
git commit -m "feat(core): end-to-end md→docx pipeline (paragraphs + h1)"
```

### Task M1-6: CI — build + test matrix

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write CI workflow**

```yaml
name: CI
on:
  push: { branches: [master, main] }
  pull_request:
jobs:
  test:
    strategy:
      matrix:
        os: [macos-14, windows-2022]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: crates/scribe-tauri/frontend/pnpm-lock.yaml }
      - run: pnpm --dir crates/scribe-tauri/frontend install --frozen-lockfile
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check
      - run: pnpm --dir crates/scribe-tauri/frontend build
```

- [ ] **Step 2: Push and verify both platforms green**

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: test + build matrix on macos + windows"
```

### Task M1-7: Sign + notarize macOS / sign Windows (debug quality bar)

**Goal:** prove the signing path end to end with a debug build. Full release signing stays in M5.

- [ ] **Step 1: Acquire Apple Developer ID Application cert (external; start paperwork now if not already)**
- [ ] **Step 2: Configure Tauri signing env vars locally (document in `docs/signing.md`)**
- [ ] **Step 3: Run `cargo tauri build`, open `.dmg` on a second Mac, verify no Gatekeeper warning**
- [ ] **Step 4: On Windows, use a self-signed cert as rehearsal (real EV cert in M5)**
- [ ] **Step 5: Commit `docs/signing.md`**

### M1 Verification Checklist

- [ ] `cargo test --workspace` green locally + CI on both platforms.
- [ ] `cargo tauri dev` launches on both platforms.
- [ ] Dropped `hello.md` produces correct `.docx`.
- [ ] macOS build signed + notarized (at least once).
- [ ] Plan reviewed against spec §4.4 data flow diagram — matches.

**Tag:** `git tag v0.1.0-m1 && git push --tags`

---

## §C. Milestone M2 — Core conversion (Week 2)

**Goal:** Full AST and DOCX emission for everything in spec §3.3 *except* math, images with sizing, and cross-refs. The `academic-en` template works end-to-end. CLI harness in place.

### M2 task list (each gets the same 5-step TDD treatment as M1-5)

- [ ] **M2-1:** Expand AST — inline formatting (bold, italic, strike, code), lists (ordered/unordered, nested), blockquotes, horizontal rules, links, task lists.
- [ ] **M2-2:** Expand parser — map all pulldown-cmark events above to AST.
- [ ] **M2-3:** Expand DOCX emitter — runs with bold/italic/underline/strike/code style, list numbering, bullet styles, blockquote indent, hyperlinks.
- [ ] **M2-4:** Code block support — `scribe-highlight` using syntect, emit styled character runs per token.
- [ ] **M2-5:** Table support in AST + parser + emitter (plain; three-line-table style is M4 via template).
- [ ] **M2-6:** Footnotes (AST, parser, emitter).
- [ ] **M2-7:** Build bundled `academic-en.docx` (in LibreOffice / Word, hand-crafted once), wire `scribe-template` to load it on startup and expose `StyleMap`.
- [ ] **M2-8:** Integrate template into emitter — headings use styleId from StyleMap, body style applied to paragraphs.
- [ ] **M2-9:** CLI harness `scribe-cli` — `scribe-cli convert input.md -o output.docx -t academic-en`. Used for all fixture tests.
- [ ] **M2-10:** Integration test suite: `fixtures/english/paper-noimages-nomath.md` round-trips to a reference `.docx` whose XML matches a normalized snapshot.

Each task: write fixture → write failing test → implement → pass → commit.

### M2 Verification Checklist

- [ ] A 10-page `.md` with headings, lists, tables, quotes, code, footnotes renders correctly in Word with `academic-en` template.
- [ ] `scribe-cli` produces the same output as the GUI.
- [ ] All integration fixtures green in CI.

**Tag:** `v0.1.0-m2`

---

## §D. Milestone M3 — Math, images, cross-refs (Week 3)

**Goal:** LaTeX math renders as OMML, images resolve + resize + caption, cross-references work for figures/tables/equations.

### M3 task list

- [ ] **M3-1:** `scribe-math::latex_to_mathml` — wrap `latex2mathml` crate, add fixture tests for the 20 most common constructs.
- [ ] **M3-2:** `scribe-math::mml_to_omml` — port Microsoft's `mml2omml.xsl` to a direct Rust transformer. Expand via fixtures iteratively. This is the hardest task in v1; budget 3 days alone. Keep the XSLT open in a tab; each XSLT template becomes a Rust function.
  - Fixtures: `\frac`, `\sqrt`, `\sum`, `\int`, matrices, `\begin{align}`, `\begin{cases}`, Greek, decorations, sub/sup, operators.
- [ ] **M3-3:** Inline math in AST + parser (pulldown-cmark Math event). Emit OMML run inline with text.
- [ ] **M3-4:** Block math (display equations). Auto-number + right-aligned `(n)`.
- [ ] **M3-5:** `scribe-images::load` — read file, detect format, extract dimensions, produce bytes suitable for embedding.
- [ ] **M3-6:** SVG → PNG via `resvg`.
- [ ] **M3-7:** Image sizing: parse extended `![alt|width=60%](img.png)` syntax. Default: fit page width.
- [ ] **M3-8:** Image captions: parse caption syntax. Auto-number "Figure N" per template locale ("Figure" / "图").
- [ ] **M3-9:** `scribe-docx::images` — insert image with correct DrawingML XML, dimensions in EMU (914400 per inch).
- [ ] **M3-10:** Cross-ref infrastructure: label registry (figure, table, equation), post-pass to resolve `[@fig:label]` into Word cross-reference fields.
- [ ] **M3-11:** Round-trip fixture: `fixtures/english/paper-full.md` (matches spec Appendix A). Full Word render check.
- [ ] **M3-12:** Round-trip fixture: `fixtures/chinese/thesis-excerpt.md` (matches spec Appendix B). Chinese-aware numbering.

### M3 Verification Checklist

- [ ] `fixtures/english/paper-full.md` opens in Word with zero manual fixes needed.
- [ ] All equations editable (not images).
- [ ] Figures numbered, captioned, cross-referenced; clicking a cross-ref jumps to figure.
- [ ] Chinese fixture uses 图 N / 表 N labels.

**Tag:** `v0.1.0-m3`

---

## §E. Milestone M4 — Templates, preview, batch (Week 4)

### M4 task list

- [ ] **M4-1:** Build `ieee.docx`, `corporate-minimal.docx`, `thesis-cn-ugrad.docx`, `journal-cn.docx` by hand in Word/LibreOffice. Source in `templates-src/`.
- [ ] **M4-2:** `scribe-template::loader` — parse `styles.xml`, extract full StyleMap (heading, body, list, code, table, quote, caption styles + default fonts + page margins + equation numbering style).
- [ ] **M4-3:** Bundled templates loaded via `include_bytes!`.
- [ ] **M4-4:** User `reference.docx` ingestion — same loader, exposed via Settings UI.
- [ ] **M4-5:** 三线表 handling — emit table with only top-border, header-bottom-border, table-bottom-border when template flag is set.
- [ ] **M4-6:** Chinese caption prefixes (图 N, 表 N) driven by template locale.
- [ ] **M4-7:** `scribe-preview::render` — AST → HTML, per-template CSS approximation.
- [ ] **M4-8:** Frontend preview pane — iframe sandbox, debounced 500ms render, MathML native rendering.
- [ ] **M4-9:** Batch mode — backend `convert_batch(input_dir, output_dir, template_id)` streaming progress events.
- [ ] **M4-10:** Frontend batch queue UI — drag folder, live progress, per-file error report.

### M4 Verification Checklist

- [ ] All 5 templates produce visually-distinct, correct output for the full fixture set.
- [ ] User-imported `reference.docx` correctly applies styles.
- [ ] Preview visually matches .docx within the ≥95% fidelity target.
- [ ] Batch: 20 files in <30 seconds on M1 MacBook.

**Tag:** `v0.1.0-m4` (feature-complete beta).

---

## §F. Milestone M5 — Polish, signing, release (Week 5)

### M5 task list

- [ ] **M5-1:** Settings UI — default template, default output dir, recent files, theme.
- [ ] **M5-2:** Persist settings/recents to `~/Library/Application Support/Folio` / `%APPDATA%\Folio`.
- [ ] **M5-3:** Drag-drop polish: multi-file, folder, visual drop states.
- [ ] **M5-4:** Toast + error UX for unsupported LaTeX macros, missing images.
- [ ] **M5-5:** "Open in Word" / "Reveal in Finder/Explorer" buttons post-convert.
- [ ] **M5-6:** App icon final artwork (stylized S; use SF Symbols + gradient — fully solo-produced in Figma or similar).
- [ ] **M5-7:** Acquire EV code-signing cert for Windows (Azure Key Vault setup).
- [ ] **M5-8:** `.github/workflows/release.yml` — signed macOS (universal) + Windows (x64 + arm64) artifacts on tag push.
- [ ] **M5-9:** Landing page (simple static site in `/website/`).
- [ ] **M5-10:** Gumroad integration for license keys (ed25519 verification in `scribe-core::license`).
- [ ] **M5-11:** Analytics: none in v1 (privacy-first). Opt-in crash reporting via Sentry deferred to v1.1.
- [ ] **M5-12:** Invite 10–20 beta testers via email; dedicated Discord or issue tracker.

### M5 Verification Checklist

- [ ] Signed .dmg + .msi install on a fresh machine with zero warnings.
- [ ] Landing page live.
- [ ] Beta tester feedback tracked.

**Tag:** `v0.1.0-rc1` when beta ready.

---

## §G. Milestone M6 — v1.0 public release (Week 6)

### M6 task list

- [ ] **M6-1:** Triage + fix P0/P1 beta bugs.
- [ ] **M6-2:** Final docs: README, user guide (`/docs/user/`), template authoring guide.
- [ ] **M6-3:** Release notes.
- [ ] **M6-4:** Announcement plan: HN, Reddit (r/rust, r/LaTeX, r/academia), 小红书, 知乎, Twitter/X.
- [ ] **M6-5:** Microsoft Store + Mac App Store submissions (separate code paths for sandboxing; can wait post-launch).

**Tag:** `v1.0.0`.

---

## §H. Cross-cutting concerns

### H.1 Versioning
- SemVer, breaking changes avoided pre-2.0.
- `CHANGELOG.md` updated every PR that ships user-visible change.

### H.2 Branching
- `main` is always shippable.
- Feature branches short-lived, squash-merged.
- Milestone tags: `v0.1.0-m1` … `v0.1.0-m6`, then `v1.0.0`.

### H.3 Testing invariants
- Every crate: unit tests.
- `scribe-core`: integration tests using fixture `.md` → `.docx` with XML normalization + diff.
- Golden file tests for OMML output, reviewed carefully on change.

### H.4 Docs
- Each public Rust module: crate-level docs + doc tests on key APIs.
- `CONTRIBUTING.md` explains how to add a template, add a math construct, regenerate fixtures.

### H.5 Release process checklist
1. All tests green on CI, both platforms.
2. `CHANGELOG.md` updated.
3. Tag pushed → release workflow builds signed artifacts.
4. DMG + MSI uploaded to GitHub Release + Gumroad.
5. Landing page version bumped.
6. Announcement posts scheduled.

---

## Appendix: Skills to reference during execution

- Use `superpowers:test-driven-development` for every task.
- Use `superpowers:verification-before-completion` before any "done" claim.
- Use `superpowers:systematic-debugging` when tests break unexpectedly.
- Use `superpowers:requesting-code-review` at the end of each milestone.
- Use `claude-api` skill is NOT applicable — this project has no LLM integration.

---

## End of plan

**Status:** Ready for execution. M1 tasks are fully detailed and start-today-actionable. M2 outlines follow the same TDD cadence. M3–M6 list their tasks and will be expanded into step-level detail at each milestone's kickoff, per YAGNI.
