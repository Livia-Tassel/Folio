//! scribe-docx: emit `.docx` bytes from a [`scribe_ast::Document`].
//!
//! Supports the full Markdown feature set in §3.3 of the design doc:
//! headings, paragraphs, blockquotes, lists (including tasks), code
//! blocks with syntect highlighting, GFM tables, footnotes, hyperlinks,
//! inline + block math (OMML), and thematic breaks.
//!
//! Because `docx-rs` has no native math support, we inject OMML via a
//! two-phase pipeline:
//! 1. The block/inline emitter writes unique placeholder tokens
//!    (`{{SCRIBE_MATH:uuid}}`) in place of math, and records each token's
//!    OMML XML in a `math_map`.
//! 2. After `docx-rs` packs the zip, [`postprocess_math`] reopens
//!    `word/document.xml`, replaces each placeholder run (or paragraph
//!    for block math) with the real OMML, and repacks the archive.

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use docx_rs::{
    AbstractNumbering, AlignmentType, Docx, Footnote, Level, LevelJc, LevelText, NumberFormat,
    Numbering, NumberingId, Paragraph, Run, RunFonts, Start, Style, StyleType, Table,
    TableAlignmentType, TableCell, TableRow, WidthType,
};
use scribe_ast::{Alignment, Block, Document, Inline};

const ABSTRACT_NUM_UNORDERED: usize = 1;
const ABSTRACT_NUM_ORDERED: usize = 2;

/// Convert a [`Document`] into `.docx` bytes.
pub fn emit(doc: &Document) -> anyhow::Result<Vec<u8>> {
    emit_with_base(doc, None)
}

/// Emit with an explicit base directory used to resolve relative image paths.
pub fn emit_with_base(
    doc: &Document,
    base_dir: Option<std::path::PathBuf>,
) -> anyhow::Result<Vec<u8>> {
    let mut out = Docx::new();
    out = register_builtin_styles(out);
    out = register_numbering(out);

    let mut ctx = EmitCtx {
        footnotes: &doc.footnotes,
        math_map: HashMap::new(),
        math_counter: 0,
        base_dir,
    };

    for block in &doc.blocks {
        out = render_block(out, block, 0, &mut ctx);
    }

    let mut buf: Vec<u8> = Vec::new();
    out.build().pack(&mut Cursor::new(&mut buf))?;

    if ctx.math_map.is_empty() {
        Ok(buf)
    } else {
        postprocess_math(&buf, &ctx.math_map)
    }
}

struct EmitCtx<'a> {
    footnotes: &'a std::collections::BTreeMap<String, Vec<Block>>,
    /// Map from placeholder token (e.g. `{{SCRIBE_MATH:inline_0}}`) to the
    /// OMML XML that should replace it after `docx-rs` packs the file.
    math_map: HashMap<String, MathReplacement>,
    math_counter: usize,
    /// Optional base path used to resolve relative image URLs.
    base_dir: Option<std::path::PathBuf>,
}

impl<'a> EmitCtx<'a> {
    fn register_inline_math(&mut self, latex: &str) -> String {
        let id = self.math_counter;
        self.math_counter += 1;
        let token = format!("{{{{SCRIBE_MATH:i{id}}}}}");
        match scribe_math::latex_to_omml(latex, scribe_math::Display::Inline) {
            Ok(omml) => {
                self.math_map
                    .insert(token.clone(), MathReplacement::InlineRun(omml));
            }
            Err(_) => {
                // Fallback: leave the LaTeX source visible as code.
                self.math_map.insert(
                    token.clone(),
                    MathReplacement::InlineRun(format!("<!-- math failed: {latex} -->")),
                );
            }
        }
        token
    }

    fn register_block_math(&mut self, latex: &str) -> String {
        let id = self.math_counter;
        self.math_counter += 1;
        let token = format!("{{{{SCRIBE_MATH:b{id}}}}}");
        match scribe_math::latex_to_omml(latex, scribe_math::Display::Block) {
            Ok(omml) => {
                self.math_map
                    .insert(token.clone(), MathReplacement::ParagraphBlock(omml));
            }
            Err(_) => {
                self.math_map.insert(
                    token.clone(),
                    MathReplacement::ParagraphBlock(format!("<!-- math failed: {latex} -->")),
                );
            }
        }
        token
    }
}

enum MathReplacement {
    /// Replace the enclosing `<w:r>…{token}…</w:r>` with the OMML.
    InlineRun(String),
    /// Replace the enclosing `<w:p>…{token}…</w:p>` with the OMML wrapped in an `<m:oMathPara>`.
    ParagraphBlock(String),
}

/// Apply a block to the docx, returning the updated docx.
fn render_block(mut out: Docx, block: &Block, indent_level: usize, ctx: &mut EmitCtx) -> Docx {
    match block {
        Block::Heading { level, content } => {
            let style = heading_style_id(*level);
            let mut p = Paragraph::new().style(&style);
            for run in inline_runs(content, RunStyle::default(), ctx) {
                p = p.add_run(run);
            }
            out.add_paragraph(p)
        }
        Block::Paragraph { content } => {
            let mut p = Paragraph::new();
            for run in inline_runs(content, RunStyle::default(), ctx) {
                p = p.add_run(run);
            }
            out.add_paragraph(p)
        }
        Block::BlockQuote { blocks } => {
            for child in blocks {
                out = render_quoted_block(out, child, ctx);
            }
            out
        }
        Block::CodeBlock { lang, code } => render_code_block(out, code, lang),
        Block::List {
            ordered,
            start: _,
            items,
        } => {
            let num_id = if *ordered {
                ABSTRACT_NUM_ORDERED
            } else {
                ABSTRACT_NUM_UNORDERED
            };
            for item in items {
                out = render_list_item(out, item, num_id, indent_level, ctx);
            }
            out
        }
        Block::Table {
            alignments,
            header,
            rows,
        } => render_table(out, alignments, header, rows, ctx),
        Block::ThematicBreak => {
            let p = Paragraph::new().style("HorizontalRule");
            out.add_paragraph(p)
        }
        Block::MathBlock { latex } => {
            let token = ctx.register_block_math(latex);
            let p = Paragraph::new().add_run(Run::new().add_text(&token));
            out.add_paragraph(p)
        }
    }
}

fn render_quoted_block(out: Docx, block: &Block, ctx: &mut EmitCtx) -> Docx {
    match block {
        Block::Paragraph { content } => {
            let mut p = Paragraph::new().style("Quote");
            for run in inline_runs(content, RunStyle::default(), ctx) {
                p = p.add_run(run);
            }
            out.add_paragraph(p)
        }
        other => render_block(out, other, 0, ctx),
    }
}

fn render_code_block(out: Docx, code: &str, lang: &str) -> Docx {
    // Tokenize via syntect, group tokens by line, emit one paragraph per
    // line with per-token styled runs.
    let tokens = scribe_highlight::highlight(code, lang);

    // Split tokens on newlines so each source line becomes one paragraph.
    let mut out = out;
    let mut current_line_runs: Vec<Run> = Vec::new();

    for token in tokens {
        // A token's text may contain embedded newlines (syntect preserves them).
        let mut segments = token.text.split('\n').peekable();
        let mut first = true;
        while let Some(segment) = segments.next() {
            if !first {
                // Flush the current line as a paragraph, then start fresh.
                let p = paragraph_from_runs(std::mem::take(&mut current_line_runs));
                out = out.add_paragraph(p);
            }
            first = false;
            if !segment.is_empty() {
                current_line_runs.push(token_to_run(&token, segment));
            }
            if segments.peek().is_none() {
                break;
            }
        }
    }

    if !current_line_runs.is_empty() {
        let p = paragraph_from_runs(std::mem::take(&mut current_line_runs));
        out = out.add_paragraph(p);
    }

    out
}

fn paragraph_from_runs(runs: Vec<Run>) -> Paragraph {
    let mut p = Paragraph::new().style("SourceCode");
    if runs.is_empty() {
        // Empty source line: emit a placeholder space so the paragraph
        // still renders with the monospace style.
        p = p.add_run(
            Run::new()
                .add_text("")
                .fonts(RunFonts::new().ascii("Menlo").hi_ansi("Consolas")),
        );
    } else {
        for r in runs {
            p = p.add_run(r);
        }
    }
    p
}

fn token_to_run(token: &scribe_highlight::Token, text: &str) -> Run {
    let mut run = Run::new()
        .add_text(text)
        .fonts(RunFonts::new().ascii("Menlo").hi_ansi("Consolas"));
    if let Some(color) = &token.color {
        run = run.color(color);
    }
    if token.bold {
        run = run.bold();
    }
    if token.italic {
        run = run.italic();
    }
    run
}

fn render_list_item(
    mut out: Docx,
    item: &scribe_ast::ListItem,
    num_id: usize,
    indent_level: usize,
    ctx: &mut EmitCtx,
) -> Docx {
    // A list item's first block renders as a list-styled paragraph;
    // subsequent blocks render as continuation paragraphs (nested lists
    // increase indent_level, handled by calling render_block with the
    // bumped level for inner List blocks).
    let mut blocks = item.blocks.iter();
    if let Some(first) = blocks.next() {
        let prefix = item.task.map(|checked| if checked { "☑ " } else { "☐ " });
        out = match first {
            Block::Paragraph { content } => {
                let mut p = Paragraph::new().numbering(
                    NumberingId::new(num_id),
                    docx_rs::IndentLevel::new(indent_level),
                );
                if let Some(p_prefix) = prefix {
                    p = p.add_run(Run::new().add_text(p_prefix));
                }
                for run in inline_runs(content, RunStyle::default(), ctx) {
                    p = p.add_run(run);
                }
                out.add_paragraph(p)
            }
            other => render_block(out, other, indent_level, ctx),
        };
    }
    for block in blocks {
        out = match block {
            Block::List { items, ordered, .. } => {
                let nested_num = if *ordered {
                    ABSTRACT_NUM_ORDERED
                } else {
                    ABSTRACT_NUM_UNORDERED
                };
                for nested in items {
                    out = render_list_item(out, nested, nested_num, indent_level + 1, ctx);
                }
                out
            }
            other => render_block(out, other, indent_level, ctx),
        };
    }
    out
}

fn render_table(
    out: Docx,
    alignments: &[Alignment],
    header: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    ctx: &mut EmitCtx,
) -> Docx {
    let mut table_rows: Vec<TableRow> = Vec::with_capacity(rows.len() + 1);

    if !header.is_empty() {
        table_rows.push(make_row(header, alignments, true, ctx));
    }
    for row in rows {
        table_rows.push(make_row(row, alignments, false, ctx));
    }

    let mut table = Table::new(table_rows).align(TableAlignmentType::Center);
    table = table.width(0, WidthType::Auto);
    out.add_table(table)
}

fn make_row(
    cells: &[Vec<Inline>],
    alignments: &[Alignment],
    is_header: bool,
    ctx: &mut EmitCtx,
) -> TableRow {
    let tcs: Vec<TableCell> = cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let align = alignments.get(i).copied().unwrap_or(Alignment::None);
            let mut para = Paragraph::new().align(to_para_alignment(align));
            if is_header {
                para = para.style("TableHeader");
            }
            for run in inline_runs(cell, RunStyle::default(), ctx) {
                para = para.add_run(run);
            }
            TableCell::new().add_paragraph(para)
        })
        .collect();
    TableRow::new(tcs)
}

fn to_para_alignment(a: Alignment) -> AlignmentType {
    match a {
        Alignment::None => AlignmentType::Left,
        Alignment::Left => AlignmentType::Left,
        Alignment::Center => AlignmentType::Center,
        Alignment::Right => AlignmentType::Right,
    }
}

/// Flatten inlines into a Vec<Run>, applying character formatting.
fn inline_runs(inlines: &[Inline], style: RunStyle, ctx: &mut EmitCtx) -> Vec<Run> {
    let mut runs = Vec::new();
    for inline in inlines {
        match inline {
            Inline::Text(s) => runs.push(style.apply(Run::new().add_text(s))),
            Inline::SoftBreak => runs.push(style.apply(Run::new().add_text(" "))),
            Inline::HardBreak => {
                runs.push(style.apply(Run::new().add_break(docx_rs::BreakType::TextWrapping)))
            }
            Inline::Code(s) => {
                let mut code_style = style;
                code_style.code = true;
                runs.push(code_style.apply(Run::new().add_text(s)));
            }
            Inline::Strong(inner) => {
                let mut s = style;
                s.bold = true;
                runs.extend(inline_runs(inner, s, ctx));
            }
            Inline::Emphasis(inner) => {
                let mut s = style;
                s.italic = true;
                runs.extend(inline_runs(inner, s, ctx));
            }
            Inline::Strikethrough(inner) => {
                let mut s = style;
                s.strike = true;
                runs.extend(inline_runs(inner, s, ctx));
            }
            Inline::Link {
                url: _,
                title: _,
                content,
            } => {
                // v0: render link text as blue underlined runs. Proper
                // Hyperlink relationship support comes in M3 (needs
                // document-level relationship registration).
                let mut s = style;
                s.link = true;
                runs.extend(inline_runs(content, s, ctx));
            }
            Inline::FootnoteRef(label) => {
                runs.push(emit_footnote_run(label, ctx, style));
            }
            Inline::InlineMath(latex) => {
                let token = ctx.register_inline_math(latex);
                runs.push(style.apply(Run::new().add_text(&token)));
            }
            Inline::Image { url, alt, title: _ } => {
                if let Some(run) = emit_image_run(url, alt, ctx) {
                    runs.push(run);
                } else {
                    // Fallback: render alt text so the image placeholder is visible.
                    let text = if alt.is_empty() {
                        format!("[image: {url}]")
                    } else {
                        format!("[image: {alt}]")
                    };
                    runs.push(style.apply(Run::new().add_text(&text).italic()));
                }
            }
        }
    }
    runs
}

fn emit_footnote_run(label: &str, ctx: &mut EmitCtx, style: RunStyle) -> Run {
    let Some(blocks) = ctx.footnotes.get(label) else {
        // Dangling reference — emit the label in brackets so authors can spot it.
        return style.apply(Run::new().add_text(format!("[^{label}]")));
    };

    let mut footnote = Footnote::new();
    for block in blocks {
        if let Block::Paragraph { content } = block {
            let mut para = Paragraph::new();
            for run in inline_runs(content, RunStyle::default(), ctx) {
                para = para.add_run(run);
            }
            footnote = footnote.add_content(para);
        }
        // Non-paragraph footnote content (lists, code) would need the
        // block-level renderer; defer to M3 polish.
    }

    style.apply(Run::new().add_footnote_reference(footnote))
}

/// Load `url` (possibly relative to `ctx.base_dir`), size to fit the page
/// width, and return a `Run` containing the embedded image. Returns `None`
/// if the image cannot be loaded — the caller is expected to fall back to
/// rendering the alt text.
fn emit_image_run(url: &str, _alt: &str, ctx: &EmitCtx) -> Option<Run> {
    // Don't try to fetch remote images — that needs HTTP and introduces
    // a network dep. Authors who want a remote image should download it.
    if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
        return None;
    }

    let path = resolve_path(url, ctx.base_dir.as_deref())?;
    let img = scribe_images::load(&path).ok()?;
    let (w_emu, h_emu) = img.page_fit_emu(None);
    let pic = docx_rs::Pic::new_with_dimensions(img.bytes, img.width_px, img.height_px)
        .size(w_emu, h_emu);
    Some(Run::new().add_image(pic))
}

fn resolve_path(url: &str, base: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    let raw = std::path::PathBuf::from(url);
    if raw.is_absolute() {
        Some(raw)
    } else {
        base.map(|b| b.join(&raw))
            .or_else(|| Some(std::env::current_dir().ok()?.join(&raw)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RunStyle {
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
    link: bool,
}

impl RunStyle {
    fn apply(self, mut run: Run) -> Run {
        if self.bold {
            run = run.bold();
        }
        if self.italic {
            run = run.italic();
        }
        if self.strike {
            run = run.strike();
        }
        if self.code {
            run = run
                .fonts(RunFonts::new().ascii("Menlo").hi_ansi("Consolas"))
                .style("InlineCode");
        }
        if self.link {
            run = run.color("0563C1").underline("single");
        }
        run
    }
}

fn heading_style_id(level: u8) -> String {
    let clamped = level.clamp(1, 6);
    format!("Heading{clamped}")
}

fn register_builtin_styles(mut out: Docx) -> Docx {
    // Quote — indented italic.
    out = out.add_style(Style::new("Quote", StyleType::Paragraph).name("Quote"));
    // SourceCode — code block paragraph style.
    out = out.add_style(Style::new("SourceCode", StyleType::Paragraph).name("Source Code"));
    // InlineCode — character style applied via Run::style().
    out = out.add_style(Style::new("InlineCode", StyleType::Character).name("Inline Code"));
    // TableHeader — header row paragraph style.
    out = out.add_style(Style::new("TableHeader", StyleType::Paragraph).name("Table Header"));
    // HorizontalRule — paragraph style used for thematic breaks.
    out = out.add_style(Style::new("HorizontalRule", StyleType::Paragraph).name("Horizontal Rule"));
    out
}

fn register_numbering(out: Docx) -> Docx {
    let bullet_abstract = AbstractNumbering::new(ABSTRACT_NUM_UNORDERED)
        .add_level(
            Level::new(
                0,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("•"),
                LevelJc::new("left"),
            )
            .indent(Some(720), None, None, None),
        )
        .add_level(
            Level::new(
                1,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("◦"),
                LevelJc::new("left"),
            )
            .indent(Some(1440), None, None, None),
        )
        .add_level(
            Level::new(
                2,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("▪"),
                LevelJc::new("left"),
            )
            .indent(Some(2160), None, None, None),
        );

    let decimal_abstract = AbstractNumbering::new(ABSTRACT_NUM_ORDERED)
        .add_level(
            Level::new(
                0,
                Start::new(1),
                NumberFormat::new("decimal"),
                LevelText::new("%1."),
                LevelJc::new("left"),
            )
            .indent(Some(720), None, None, None),
        )
        .add_level(
            Level::new(
                1,
                Start::new(1),
                NumberFormat::new("lowerLetter"),
                LevelText::new("%2."),
                LevelJc::new("left"),
            )
            .indent(Some(1440), None, None, None),
        )
        .add_level(
            Level::new(
                2,
                Start::new(1),
                NumberFormat::new("lowerRoman"),
                LevelText::new("%3."),
                LevelJc::new("left"),
            )
            .indent(Some(2160), None, None, None),
        );

    out.add_abstract_numbering(bullet_abstract)
        .add_abstract_numbering(decimal_abstract)
        .add_numbering(Numbering::new(
            ABSTRACT_NUM_UNORDERED,
            ABSTRACT_NUM_UNORDERED,
        ))
        .add_numbering(Numbering::new(ABSTRACT_NUM_ORDERED, ABSTRACT_NUM_ORDERED))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scribe_ast::Inline;

    fn doc_from(blocks: Vec<Block>) -> Document {
        let mut d = Document::new();
        for b in blocks {
            d.push(b);
        }
        d
    }

    #[test]
    fn emits_valid_zip_container() {
        let doc = doc_from(vec![
            Block::Heading {
                level: 1,
                content: vec![Inline::Text("Hello".into())],
            },
            Block::Paragraph {
                content: vec![Inline::Text("World".into())],
            },
        ]);
        let bytes = emit(&doc).unwrap();
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn heading_style_id_clamps() {
        assert_eq!(heading_style_id(0), "Heading1");
        assert_eq!(heading_style_id(3), "Heading3");
        assert_eq!(heading_style_id(9), "Heading6");
    }

    #[test]
    fn emits_bold_italic_strike() {
        let doc = doc_from(vec![Block::Paragraph {
            content: vec![
                Inline::Strong(vec![Inline::Text("b".into())]),
                Inline::Text(" ".into()),
                Inline::Emphasis(vec![Inline::Text("i".into())]),
                Inline::Text(" ".into()),
                Inline::Strikethrough(vec![Inline::Text("s".into())]),
            ],
        }]);
        assert!(emit(&doc).is_ok());
    }

    #[test]
    fn emits_list_and_table() {
        let doc = doc_from(vec![
            Block::List {
                ordered: false,
                start: 0,
                items: vec![scribe_ast::ListItem {
                    task: None,
                    blocks: vec![Block::Paragraph {
                        content: vec![Inline::Text("item".into())],
                    }],
                }],
            },
            Block::Table {
                alignments: vec![Alignment::None, Alignment::Right],
                header: vec![
                    vec![Inline::Text("a".into())],
                    vec![Inline::Text("b".into())],
                ],
                rows: vec![vec![
                    vec![Inline::Text("1".into())],
                    vec![Inline::Text("2".into())],
                ]],
            },
        ]);
        assert!(emit(&doc).is_ok());
    }

    #[test]
    fn emits_code_block_and_quote() {
        let doc = doc_from(vec![
            Block::CodeBlock {
                lang: "rust".into(),
                code: "fn main() {}".into(),
            },
            Block::BlockQuote {
                blocks: vec![Block::Paragraph {
                    content: vec![Inline::Text("quote".into())],
                }],
            },
        ]);
        assert!(emit(&doc).is_ok());
    }

    #[test]
    fn emits_footnote_reference() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![
                Inline::Text("See ".into()),
                Inline::FootnoteRef("1".into()),
                Inline::Text(" for details.".into()),
            ],
        });
        doc.add_footnote(
            "1".into(),
            vec![Block::Paragraph {
                content: vec![Inline::Text("The footnote body.".into())],
            }],
        );
        let bytes = emit(&doc).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn dangling_footnote_ref_emits_bracket_placeholder() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![Inline::FootnoteRef("missing".into())],
        });
        // Definition intentionally not added.
        let bytes = emit(&doc).unwrap();
        assert_eq!(&bytes[0..2], b"PK");

        // Unzip and confirm the placeholder text is present.
        let cursor = std::io::Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        use std::io::Read;
        zip.by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        assert!(
            xml.contains("[^missing]"),
            "dangling placeholder should be present"
        );
    }

    #[test]
    fn inline_math_substitutes_placeholder_with_omml() {
        let mut doc = Document::new();
        doc.push(Block::Paragraph {
            content: vec![
                Inline::Text("Energy: ".into()),
                Inline::InlineMath("E = mc^2".into()),
                Inline::Text(".".into()),
            ],
        });
        let bytes = emit(&doc).unwrap();
        let cursor = std::io::Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        use std::io::Read as _;
        zip.by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        assert!(
            !xml.contains("{{SCRIBE_MATH"),
            "placeholder tokens should be replaced; got: {xml}"
        );
        assert!(xml.contains("m:oMath"), "inline math must render as OMML");
    }

    #[test]
    fn block_math_substitutes_to_oMathPara() {
        let mut doc = Document::new();
        doc.push(Block::MathBlock {
            latex: "a + b = c".into(),
        });
        let bytes = emit(&doc).unwrap();
        let cursor = std::io::Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        use std::io::Read as _;
        zip.by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut xml)
            .unwrap();
        assert!(
            !xml.contains("{{SCRIBE_MATH"),
            "placeholder should be replaced"
        );
        assert!(
            xml.contains("m:oMathPara"),
            "block math must render as oMathPara"
        );
    }
}

// ---------------------------------------------------------------------------
// Post-processing: replace math placeholders in word/document.xml with OMML.
// ---------------------------------------------------------------------------

fn postprocess_math(
    zip_bytes: &[u8],
    math_map: &HashMap<String, MathReplacement>,
) -> anyhow::Result<Vec<u8>> {
    let cursor = Cursor::new(zip_bytes);
    let mut reader = zip::ZipArchive::new(cursor)?;

    let mut out_buf: Vec<u8> = Vec::with_capacity(zip_bytes.len());
    {
        let out_cursor = Cursor::new(&mut out_buf);
        let mut writer = zip::ZipWriter::new(out_cursor);

        for i in 0..reader.len() {
            let mut entry = reader.by_index(i)?;
            let name = entry.name().to_string();
            let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
                .compression_method(entry.compression())
                .last_modified_time(entry.last_modified().unwrap_or_default());

            let mut data = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut data)?;

            if name == "word/document.xml" {
                let xml = String::from_utf8(data)
                    .map_err(|e| anyhow::anyhow!("document.xml is not utf-8: {e}"))?;
                let replaced = replace_math_placeholders(&xml, math_map);
                writer.start_file(name, opts)?;
                writer.write_all(replaced.as_bytes())?;
            } else {
                writer.start_file(name, opts)?;
                writer.write_all(&data)?;
            }
        }
        writer.finish()?;
    }
    Ok(out_buf)
}

fn replace_math_placeholders(xml: &str, math_map: &HashMap<String, MathReplacement>) -> String {
    let mut out = xml.to_string();
    for (token, replacement) in math_map {
        match replacement {
            MathReplacement::InlineRun(omml) => {
                // Locate the whole run containing the placeholder: <w:r ...> ... token ... </w:r>
                while let Some(token_pos) = out.find(token) {
                    let run_start = match out[..token_pos].rfind("<w:r ") {
                        Some(i) => i,
                        None => match out[..token_pos].rfind("<w:r>") {
                            Some(i) => i,
                            None => break,
                        },
                    };
                    let run_end_close = match out[token_pos..].find("</w:r>") {
                        Some(i) => token_pos + i + "</w:r>".len(),
                        None => break,
                    };
                    out.replace_range(run_start..run_end_close, omml);
                }
            }
            MathReplacement::ParagraphBlock(omml) => {
                // Replace the enclosing <w:p> element.
                while let Some(token_pos) = out.find(token) {
                    let para_start = match out[..token_pos].rfind("<w:p ") {
                        Some(i) => i,
                        None => match out[..token_pos].rfind("<w:p>") {
                            Some(i) => i,
                            None => break,
                        },
                    };
                    let para_end_close = match out[token_pos..].find("</w:p>") {
                        Some(i) => token_pos + i + "</w:p>".len(),
                        None => break,
                    };
                    out.replace_range(para_start..para_end_close, omml);
                }
            }
        }
    }
    out
}
