# Scribe M2 Kitchen Sink

A comprehensive sample covering the full M2 feature set.

## Inline formatting

Here is **bold** and *italic* and ~~strikethrough~~ and `inline code`. You can
also [link to Scribe's repo](https://example.com "tooltip").

## Lists

Unordered:

- apples
- bananas
- cherries

Ordered (starting at 3):

3. three
4. four
5. five

Task list:

- [x] design spec
- [x] plan
- [ ] ship v1.0

## Code blocks

```rust
fn main() {
    println!("Hello, world!");
}
```

## Blockquote

> The goal is a .docx that opens in Word without any manual cleanup.
> The user never has to touch a ruler or a style dropdown.

## Table

| Feature     | Status | Notes                        |
| :---------- | :----: | ---------------------------: |
| Headings    |   ✅   | H1–H6                        |
| Inlines     |   ✅   | bold, italic, code, link     |
| Tables      |   ✅   | GFM with alignment           |
| Footnotes   |   ✅   | native Word footnotes        |
| Math        |   ⏳   | coming in M3                 |

---

## Footnotes

Scribe also supports footnotes[^fn-a] that appear at the bottom of the page.
Multiple footnotes per document[^fn-b] are fine too.

[^fn-a]: Footnotes are rendered as native Word footnotes, editable just like any
    reference inserted through Word's own UI.

[^fn-b]: Each footnote gets its own auto-assigned ID.

End of sample.
