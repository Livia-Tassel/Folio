# Folio Demo Document

> Folio compiles Markdown directly into **native Word structures** — editable equations, well-formed tables, aligned fonts, with no manual cleanup pass.

## Heading hierarchy

The next paragraphs show the visual difference between H2, H3, and H4.

### Heading 3: research context

Academic writers and technical authors share one pain point — **formatting cleanup**. After the content is done, half the day is spent dragging styles from Markdown into Word. Folio is designed to eliminate that step.

#### Heading 4: implementation note

Folio uses a pure-Rust pipeline — no Pandoc dependency, no Python runtime required for the CLI or desktop app. Equations are emitted as native OMML, not images.

## Inline formatting

A paragraph with a forced line break here:\
The second sentence continues on the next line inside the same paragraph.

We support **bold**, *italic*, ~~strikethrough~~, `inline code`, [hyperlinks](https://github.com/Livia-Tassel/Folio), and inline math like $E = mc^2$.

Mixed scripts: **Markdown to Word** receives over 10,000 monthly searches, yet existing tools fail on *equation rendering* almost universally.

## Equations

Inline math: when $a \ne 0$, the quadratic $ax^2 + bx + c = 0$ has the solutions below.

Display math:

$$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

Einstein's mass–energy equivalence:

$$E = mc^2$$

Sums and integrals:

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2} \qquad \int_{0}^{1} x^2 \, dx = \frac{1}{3}$$

A matrix:

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix}$$

## Code blocks

Rust with syntax highlighting:

```rust
fn main() {
    let greeting = "Hello";
    println!("{}, Folio!", greeting);
}
```

Python example:

```python
import folio

data = folio.convert("# Hello\n\n$E = mc^2$", theme="academic")
with open("paper.docx", "wb") as f:
    f.write(data)
```

JSON config:

```json
{
  "tool": "Folio",
  "version": "0.2.1",
  "features": ["math", "tables", "images", "footnotes"]
}
```

## Tables

Aligned table (left / center / right columns):

| Theme | Font | Use case |
| :---- | :---: | -----: |
| `academic` | Times New Roman 12pt | English academic papers |
| `thesis-cn` | SimSun + SimHei headings | Chinese theses |
| `report` | Calibri with blue accents | Business memos |

## Lists and tasks

Unordered list with nesting:

- Command-line (`scribe-cli`)
  - Convert a single file
  - Batch convert with a shell loop
- Python package (`folio-docx`)
  - `folio.convert()` — string to bytes
  - `folio.convert_file()` — file to file
- Desktop application (Tauri)

Ordered list:

1. Write Markdown
2. Pick a theme or supply a reference doc
3. Export to Word — **no manual cleanup**

Task list:

- [x] Markdown → native Word structures
- [x] Built-in themes
- [x] Reference-doc inherits styles + page setup
- [ ] Custom theme scaffolder (planned)

## Images

App icon (raster PNG):

![Folio app icon](assets/folio-icon.png "Folio app icon")

Wide banner (SVG, auto-scaled to page width):

![Folio banner](assets/folio-banner.svg "Folio banner")

Layout diagram (SVG):

![Folio layout diagram](assets/layout-diagram.svg "Folio layout diagram")

## Quotes and footnotes

> "Markdown to polished `.docx` output, without the cleanup pass."
>
> — Folio project page

Body text with a footnote[^1], expanded at the document end.

[^1]: Footnotes become native Word footnotes — not in-page links — so they survive copy-paste into other Word documents.

## Horizontal rule

The line below is `---`:

---

End of demo. Try it:

```bash
scribe-cli demo-en.md -o demo-en.docx --theme academic
```
