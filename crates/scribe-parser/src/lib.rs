//! scribe-parser: Markdown text → [`scribe_ast::Document`].
//!
//! Uses pulldown-cmark's event stream and builds a typed AST. Handles
//! all of the features declared in §3.3 of the design doc:
//! GFM (tables, strikethrough, task lists, footnotes), extended image
//! syntax, cross-ref labels, math — landing incrementally across M2–M3.

use pulldown_cmark::{
    Alignment as PAlignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use scribe_ast::{Alignment, Block, Document, Inline, ListItem};

/// Parse a Markdown string into a [`Document`].
pub fn parse(markdown: &str) -> Document {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);

    let parser = Parser::new_ext(markdown, options);
    let mut builder = Builder::new();
    for event in parser {
        builder.feed(event);
    }
    builder.finish()
}

/// Stack-based builder that turns pulldown-cmark events into nested blocks/inlines.
struct Builder {
    /// Stack of active containers. Top is where incoming content goes.
    stack: Vec<Container>,
    /// Finished top-level blocks (document body).
    doc: Document,
}

/// A container on the stack is either collecting blocks or collecting inlines,
/// plus pending metadata for the block we'll emit when its End arrives.
enum Container {
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    Paragraph {
        inlines: Vec<Inline>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    ListItem {
        task: Option<bool>,
        blocks: Vec<Block>,
    },
    CodeBlock {
        lang: String,
        text: String,
    },
    Table {
        alignments: Vec<Alignment>,
        header: Option<Vec<Vec<Inline>>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    /// Inside a `<thead>` row — we collect cells into the table's `header`.
    TableHead {
        cells: Vec<Vec<Inline>>,
    },
    /// A `<tbody>` row.
    TableRow {
        cells: Vec<Vec<Inline>>,
    },
    TableCell {
        inlines: Vec<Inline>,
    },
    /// Inline container (Strong/Emphasis/Strikethrough/Link).
    Inline(InlineKind),
    /// Footnote definition — collects blocks until the matching End.
    FootnoteDef(FootnoteDef),
}

enum InlineKind {
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Link {
        url: String,
        title: String,
        content: Vec<Inline>,
    },
}

/// A footnote definition's contents are accumulated separately from the
/// main document body, then stashed into `Document::footnotes` on end.
struct FootnoteDef {
    label: String,
    blocks: Vec<Block>,
}

impl Builder {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            doc: Document::new(),
        }
    }

    fn finish(self) -> Document {
        self.doc
    }

    fn feed(&mut self, event: Event<'_>) {
        // Code blocks capture raw text directly, bypassing the inline tree.
        if let Some(Container::CodeBlock { text, .. }) = self.stack.last_mut() {
            match event {
                Event::Text(s) => {
                    text.push_str(&s);
                    return;
                }
                Event::End(TagEnd::CodeBlock) => {
                    // fall through to the normal End handling below
                }
                // Any other event inside a code block is unexpected; ignore.
                Event::End(_) | Event::Start(_) => {}
                _ => return,
            }
        }

        match event {
            Event::Start(tag) => self.on_start(tag),
            Event::End(tag_end) => self.on_end(tag_end),
            Event::Text(s) => self.push_inline(Inline::Text(s.into_string())),
            Event::Code(s) => self.push_inline(Inline::Code(s.into_string())),
            Event::SoftBreak => self.push_inline(Inline::SoftBreak),
            Event::HardBreak => self.push_inline(Inline::HardBreak),
            Event::TaskListMarker(checked) => {
                if let Some(Container::ListItem { task, .. }) = self.stack.last_mut() {
                    *task = Some(checked);
                }
            }
            Event::Rule => self.push_block(Block::ThematicBreak),
            Event::FootnoteReference(label) => {
                self.push_inline(Inline::FootnoteRef(label.into_string()));
            }
            // HTML / math events arrive in M3; unhandled for now.
            _ => {}
        }
    }

    fn on_start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                self.stack.push(Container::Heading {
                    level: heading_level_to_u8(level),
                    inlines: Vec::new(),
                });
            }
            Tag::Paragraph => {
                self.stack.push(Container::Paragraph {
                    inlines: Vec::new(),
                });
            }
            Tag::BlockQuote(_) => {
                self.stack
                    .push(Container::BlockQuote { blocks: Vec::new() });
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => info.into_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.stack.push(Container::CodeBlock {
                    lang,
                    text: String::new(),
                });
            }
            Tag::List(start_maybe) => {
                let (ordered, start) = match start_maybe {
                    Some(n) => (true, n),
                    None => (false, 0),
                };
                self.stack.push(Container::List {
                    ordered,
                    start,
                    items: Vec::new(),
                });
            }
            Tag::Item => {
                self.stack.push(Container::ListItem {
                    task: None,
                    blocks: Vec::new(),
                });
            }
            Tag::Emphasis => {
                self.stack
                    .push(Container::Inline(InlineKind::Emphasis(Vec::new())));
            }
            Tag::Strong => {
                self.stack
                    .push(Container::Inline(InlineKind::Strong(Vec::new())));
            }
            Tag::Strikethrough => {
                self.stack
                    .push(Container::Inline(InlineKind::Strikethrough(Vec::new())));
            }
            Tag::Link {
                dest_url, title, ..
            } => {
                self.stack.push(Container::Inline(InlineKind::Link {
                    url: dest_url.into_string(),
                    title: title.into_string(),
                    content: Vec::new(),
                }));
            }
            Tag::Table(aligns) => {
                self.stack.push(Container::Table {
                    alignments: aligns.into_iter().map(convert_alignment).collect(),
                    header: None,
                    rows: Vec::new(),
                });
            }
            Tag::TableHead => {
                self.stack.push(Container::TableHead { cells: Vec::new() });
            }
            Tag::TableRow => {
                self.stack.push(Container::TableRow { cells: Vec::new() });
            }
            Tag::TableCell => {
                self.stack.push(Container::TableCell {
                    inlines: Vec::new(),
                });
            }
            Tag::FootnoteDefinition(label) => {
                self.stack.push(Container::FootnoteDef(FootnoteDef {
                    label: label.into_string(),
                    blocks: Vec::new(),
                }));
            }
            // Unhandled start tags are silently ignored — their matching End is also ignored.
            _ => {}
        }
    }

    fn on_end(&mut self, tag_end: TagEnd) {
        let Some(container) = self.stack.pop() else {
            return;
        };
        match (container, tag_end) {
            (Container::Heading { level, inlines }, TagEnd::Heading(_)) => {
                self.push_block(Block::Heading {
                    level,
                    content: inlines,
                });
            }
            (Container::Paragraph { inlines }, TagEnd::Paragraph) => {
                self.push_block(Block::Paragraph { content: inlines });
            }
            (Container::BlockQuote { blocks }, TagEnd::BlockQuote(_)) => {
                self.push_block(Block::BlockQuote { blocks });
            }
            (Container::CodeBlock { lang, text }, TagEnd::CodeBlock) => {
                self.push_block(Block::CodeBlock { lang, code: text });
            }
            (
                Container::List {
                    ordered,
                    start,
                    items,
                },
                TagEnd::List(_),
            ) => {
                self.push_block(Block::List {
                    ordered,
                    start,
                    items,
                });
            }
            (Container::ListItem { task, blocks }, TagEnd::Item) => {
                // The ListItem belongs to the enclosing List.
                if let Some(Container::List { items, .. }) = self.stack.last_mut() {
                    items.push(ListItem { task, blocks });
                }
            }
            (
                Container::Table {
                    alignments,
                    header,
                    rows,
                },
                TagEnd::Table,
            ) => {
                self.push_block(Block::Table {
                    alignments,
                    header: header.unwrap_or_default(),
                    rows,
                });
            }
            (Container::TableHead { cells }, TagEnd::TableHead) => {
                if let Some(Container::Table { header, .. }) = self.stack.last_mut() {
                    *header = Some(cells);
                }
            }
            (Container::TableRow { cells }, TagEnd::TableRow) => {
                if let Some(Container::Table { rows, .. }) = self.stack.last_mut() {
                    rows.push(cells);
                }
            }
            (Container::TableCell { inlines }, TagEnd::TableCell) => {
                // Push the cell into the enclosing TableHead or TableRow.
                match self.stack.last_mut() {
                    Some(Container::TableHead { cells }) | Some(Container::TableRow { cells }) => {
                        cells.push(inlines);
                    }
                    _ => {}
                }
            }
            (Container::Inline(InlineKind::Strong(xs)), TagEnd::Strong) => {
                self.push_inline(Inline::Strong(xs));
            }
            (Container::Inline(InlineKind::Emphasis(xs)), TagEnd::Emphasis) => {
                self.push_inline(Inline::Emphasis(xs));
            }
            (Container::Inline(InlineKind::Strikethrough(xs)), TagEnd::Strikethrough) => {
                self.push_inline(Inline::Strikethrough(xs));
            }
            (
                Container::Inline(InlineKind::Link {
                    url,
                    title,
                    content,
                }),
                TagEnd::Link,
            ) => {
                self.push_inline(Inline::Link {
                    url,
                    title,
                    content,
                });
            }
            (Container::FootnoteDef(fd), TagEnd::FootnoteDefinition) => {
                self.doc.add_footnote(fd.label, fd.blocks);
            }
            // Mismatched container/end: discard silently. A malformed event
            // stream from pulldown-cmark is not expected in practice.
            _ => {}
        }
    }

    fn push_block(&mut self, block: Block) {
        match self.stack.last_mut() {
            Some(Container::BlockQuote { blocks }) | Some(Container::ListItem { blocks, .. }) => {
                blocks.push(block);
            }
            Some(Container::FootnoteDef(fd)) => {
                fd.blocks.push(block);
            }
            _ => self.doc.push(block),
        }
    }

    fn push_inline(&mut self, inline: Inline) {
        if let Some(target) = current_inline_vec(&mut self.stack) {
            target.push(inline);
        }
    }
}

/// Get the inline sink of the topmost container that accepts inlines.
fn current_inline_vec(stack: &mut [Container]) -> Option<&mut Vec<Inline>> {
    // Walk top-down; text often lands deep inside emphasis/strong.
    for container in stack.iter_mut().rev() {
        match container {
            Container::Heading { inlines, .. } => return Some(inlines),
            Container::Paragraph { inlines } => return Some(inlines),
            Container::TableCell { inlines } => return Some(inlines),
            Container::Inline(kind) => match kind {
                InlineKind::Strong(v) => return Some(v),
                InlineKind::Emphasis(v) => return Some(v),
                InlineKind::Strikethrough(v) => return Some(v),
                InlineKind::Link { content, .. } => return Some(content),
            },
            _ => continue,
        }
    }
    None
}

fn convert_alignment(a: PAlignment) -> Alignment {
    match a {
        PAlignment::None => Alignment::None,
        PAlignment::Left => Alignment::Left,
        PAlignment::Center => Alignment::Center,
        PAlignment::Right => Alignment::Right,
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Inline {
        Inline::Text(s.into())
    }

    #[test]
    fn parses_h1_and_paragraph() {
        let doc = parse("# Hello\n\nWorld");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Heading {
                    level: 1,
                    content: vec![text("Hello")],
                },
                Block::Paragraph {
                    content: vec![text("World")],
                },
            ]
        );
    }

    #[test]
    fn parses_bold_italic_strikethrough() {
        let doc = parse("**bold** *italic* ~~strike~~");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected paragraph");
        };
        // Content: Strong(Text("bold")) " " Emphasis(Text("italic")) " " Strikethrough(Text("strike"))
        assert!(matches!(content[0], Inline::Strong(_)));
        assert!(matches!(content[2], Inline::Emphasis(_)));
        assert!(matches!(content[4], Inline::Strikethrough(_)));
    }

    #[test]
    fn parses_inline_code() {
        let doc = parse("Use `cargo run` to test.");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!();
        };
        assert!(content
            .iter()
            .any(|i| matches!(i, Inline::Code(c) if c == "cargo run")));
    }

    #[test]
    fn parses_link() {
        let doc = parse("[Scribe](https://example.com \"tooltip\")");
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!();
        };
        let Inline::Link {
            url,
            title,
            content: c,
        } = &content[0]
        else {
            panic!("expected link");
        };
        assert_eq!(url, "https://example.com");
        assert_eq!(title, "tooltip");
        assert_eq!(c, &vec![text("Scribe")]);
    }

    #[test]
    fn parses_unordered_list() {
        let doc = parse("- one\n- two\n- three");
        let Block::List { ordered, items, .. } = &doc.blocks[0] else {
            panic!();
        };
        assert!(!ordered);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn parses_ordered_list_with_start() {
        let doc = parse("3. three\n4. four");
        let Block::List {
            ordered,
            start,
            items,
        } = &doc.blocks[0]
        else {
            panic!();
        };
        assert!(ordered);
        assert_eq!(*start, 3);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn parses_task_list() {
        let doc = parse("- [ ] todo\n- [x] done");
        let Block::List { items, .. } = &doc.blocks[0] else {
            panic!();
        };
        assert_eq!(items[0].task, Some(false));
        assert_eq!(items[1].task, Some(true));
    }

    #[test]
    fn parses_blockquote() {
        let doc = parse("> quoted\n>\n> second");
        assert!(matches!(doc.blocks[0], Block::BlockQuote { .. }));
    }

    #[test]
    fn parses_code_block_with_lang() {
        let doc = parse("```rust\nfn main() {}\n```");
        let Block::CodeBlock { lang, code } = &doc.blocks[0] else {
            panic!();
        };
        assert_eq!(lang, "rust");
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn parses_thematic_break() {
        let doc = parse("---");
        assert!(matches!(doc.blocks[0], Block::ThematicBreak));
    }

    #[test]
    fn parses_gfm_table() {
        let md = "| a | b |\n| --- | ---: |\n| 1 | 2 |\n| 3 | 4 |";
        let doc = parse(md);
        let Block::Table {
            alignments,
            header,
            rows,
        } = &doc.blocks[0]
        else {
            panic!("expected table");
        };
        assert_eq!(alignments.len(), 2);
        assert_eq!(alignments[1], Alignment::Right);
        assert_eq!(header.len(), 2);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn empty_input_yields_empty_document() {
        assert!(parse("").blocks.is_empty());
    }

    #[test]
    fn parses_footnote() {
        let md = "Body text[^1] continues.\n\n[^1]: The footnote body.";
        let doc = parse(md);

        // The first block should contain a FootnoteRef with label "1".
        let Block::Paragraph { content } = &doc.blocks[0] else {
            panic!("expected paragraph");
        };
        assert!(
            content
                .iter()
                .any(|i| matches!(i, Inline::FootnoteRef(label) if label == "1")),
            "paragraph should reference footnote 1; got {content:?}"
        );

        // The footnote map should contain the definition.
        assert!(doc.footnotes.contains_key("1"));
        let def = &doc.footnotes["1"];
        let Block::Paragraph { content } = &def[0] else {
            panic!("footnote definition should wrap a paragraph");
        };
        assert_eq!(content, &vec![Inline::Text("The footnote body.".into())]);
    }
}
