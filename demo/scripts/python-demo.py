"""Folio Python demo script — v0.2.1

This file is structured as a series of `# %%` cells so you can either:

  - run it top-to-bottom as a regular script:
        python demo/scripts/python-demo.py

  - paste blocks into a Jupyter notebook / IPython REPL one at a time
    while recording (each `# %%` is a notebook cell boundary in
    VSCode / PyCharm / Jupytext)

Prerequisite (one-time):
    pip install folio-docx
"""

from __future__ import annotations
import os

# ---------------------------------------------------------------------------
# %% Cell 1 — verify install
# ---------------------------------------------------------------------------
import folio

print("folio version:", folio.__version__)
print("built-in themes:", folio.list_themes())


# ---------------------------------------------------------------------------
# %% Cell 2 — Markdown string → .docx bytes (default styles)
# ---------------------------------------------------------------------------
markdown = """# Hello, Folio

Inline math: $E = mc^2$

Display math:

$$\\int_{0}^{1} x^2 \\, dx = \\frac{1}{3}$$

| Theme | Use case |
|---|---|
| academic | English papers |
| thesis-cn | 中文论文 |
| report | Business memos |
"""

data = folio.convert(markdown)
print(f"output is {len(data):,} bytes; DOCX magic = {data[:2]!r}")

os.makedirs("demo/outputs", exist_ok=True)
with open("demo/outputs/python-default.docx", "wb") as f:
    f.write(data)
print("→ wrote demo/outputs/python-default.docx")


# ---------------------------------------------------------------------------
# %% Cell 3 — built-in themes
# ---------------------------------------------------------------------------
for theme in folio.list_themes():
    out = f"demo/outputs/python-{theme}.docx"
    themed = folio.convert(markdown, theme=theme)
    with open(out, "wb") as f:
        f.write(themed)
    print(f"→ wrote {out} ({theme} theme, {len(themed):,} bytes)")


# ---------------------------------------------------------------------------
# %% Cell 4 — Chinese content + thesis-cn theme
# ---------------------------------------------------------------------------
cn_md = """# 引言

本论文研究 Markdown 到 Word 转换的工程实践。

行内公式：当 $a \\ne 0$，一元二次方程 $ax^2 + bx + c = 0$ 有以下解。

## 公式

$$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$$

## 代码

```python
import folio
data = folio.convert("# 你好\\n\\n$E = mc^2$", theme="thesis-cn")
```

## 表格

| 章节 | 内容 |
|---|---|
| 第一章 | 引言 |
| 第二章 | 方法 |
| 第三章 | 实验结果 |
"""

data = folio.convert(cn_md, theme="thesis-cn")
with open("demo/outputs/python-thesis-cn-custom.docx", "wb") as f:
    f.write(data)
print("→ wrote demo/outputs/python-thesis-cn-custom.docx (中文论文样式)")


# ---------------------------------------------------------------------------
# %% Cell 5 — file-to-file conversion (images are resolved relative to input)
# ---------------------------------------------------------------------------
folio.convert_file(
    "demo/demo-en.md",
    "demo/outputs/python-file-academic.docx",
    theme="academic",
)
print("→ wrote demo/outputs/python-file-academic.docx (file-to-file, with images)")

folio.convert_file(
    "demo/demo-cn.md",
    "demo/outputs/python-file-thesis-cn.docx",
    theme="thesis-cn",
)
print("→ wrote demo/outputs/python-file-thesis-cn.docx (中文, file-to-file, with images)")


# ---------------------------------------------------------------------------
# %% Cell 6 — conference / journal templates (reference-doc)
# ---------------------------------------------------------------------------
templates_dir = "demo/templates"
templates = sorted(
    os.path.join(templates_dir, f)
    for f in os.listdir(templates_dir)
    if f.endswith(".docx")
) if os.path.isdir(templates_dir) else []

if templates:
    for tmpl in templates:
        name = os.path.splitext(os.path.basename(tmpl))[0]
        out = f"demo/outputs/python-ref-{name}.docx"
        folio.convert_file("demo/demo-en.md", out, reference_doc=tmpl)
        print(f"→ wrote {out} (template: {name})")
else:
    print("(skip — place .docx files in demo/templates/ to demo reference-doc)")


# ---------------------------------------------------------------------------
# %% Cell 7 — mutual-exclusion guard rail
# ---------------------------------------------------------------------------
try:
    folio.convert(
        "# test",
        theme="academic",
        reference_doc=templates[0] if templates else "nonexistent.docx",
    )
except ValueError as e:
    print(f"got expected ValueError: {e}")


# ---------------------------------------------------------------------------
# %% Cell 8 — HTML preview (same renderer as the desktop app)
# ---------------------------------------------------------------------------
fragment = folio.preview_html("# Heading\n\n**bold** and $E = mc^2$")
print("preview_html fragment (first 200 chars):")
print(fragment[:200])

standalone = folio.preview_standalone("# Full Page\n\n*italic* and $\\pi \\approx 3.14$")
print("\npreview_standalone starts with:", standalone[:60])


# ---------------------------------------------------------------------------
# %% Cell 9 — concurrent batch conversion (GIL released)
# ---------------------------------------------------------------------------
from concurrent.futures import ThreadPoolExecutor
import time

batch = [
    f"# Document {i}\n\nBody text for doc {i}, with $x_{{{i}}} = {i}^2$."
    for i in range(20)
]

start = time.perf_counter()
with ThreadPoolExecutor(max_workers=8) as pool:
    outputs = list(pool.map(folio.convert, batch))
elapsed = time.perf_counter() - start

print(f"batch: converted {len(outputs)} docs in {elapsed:.3f}s (8 threads)")
print(f"all valid docx: {all(o[:2] == b'PK' for o in outputs)}")


# ---------------------------------------------------------------------------
# %% Cell 10 — summary
# ---------------------------------------------------------------------------
print()
print("Outputs written under demo/outputs/:")
for fname in sorted(os.listdir("demo/outputs")):
    full = os.path.join("demo/outputs", fname)
    size = os.path.getsize(full)
    print(f"  {fname:<45}  {size:>9,} bytes")
