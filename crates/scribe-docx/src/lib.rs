//! scribe-docx: emit `.docx` bytes from a [`scribe_ast::Document`].
//!
//! v0.2 (M2): supports the full set of Markdown features in §3.3 of the
//! design doc except math (M3) and images (M3).

use std::io::Cursor;

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
    let mut out = Docx::new();
    out = register_builtin_styles(out);
    out = register_numbering(out);

    let ctx = EmitCtx {
        footnotes: &doc.footnotes,
    };

    for block in &doc.blocks {
        out = render_block(out, block, 0, &ctx);
    }

    let mut buf: Vec<u8> = Vec::new();
    out.build().pack(&mut Cursor::new(&mut buf))?;
    Ok(buf)
}

struct EmitCtx<'a> {
    footnotes: &'a std::collections::BTreeMap<String, Vec<Block>>,
}

/// Apply a block to the docx, returning the updated docx.
fn render_block(mut out: Docx, block: &Block, indent_level: usize, ctx: &EmitCtx) -> Docx {
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
    }
}

fn render_quoted_block(out: Docx, block: &Block, ctx: &EmitCtx) -> Docx {
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
    ctx: &EmitCtx,
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
    ctx: &EmitCtx,
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
    ctx: &EmitCtx,
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
fn inline_runs(inlines: &[Inline], style: RunStyle, ctx: &EmitCtx) -> Vec<Run> {
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
        }
    }
    runs
}

fn emit_footnote_run(label: &str, ctx: &EmitCtx, style: RunStyle) -> Run {
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
}
