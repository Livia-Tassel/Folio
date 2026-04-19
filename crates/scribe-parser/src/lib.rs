//! scribe-parser: Markdown text → [`scribe_ast::Document`].
//!
//! Minimal v0: handles H1–H6 and paragraphs only. M2 expands to the full
//! GFM feature set specified in the design doc.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use scribe_ast::{Block, Document};

/// Parse a Markdown string into a [`Document`].
pub fn parse(markdown: &str) -> Document {
    let mut doc = Document::new();
    let mut buffer = String::new();
    let mut mode: Option<Mode> = None;

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                mode = Some(Mode::Heading(heading_level_to_u8(level)));
                buffer.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(Mode::Heading(level)) = mode.take() {
                    doc.push(Block::Heading {
                        level,
                        text: std::mem::take(&mut buffer),
                    });
                }
            }
            Event::Start(Tag::Paragraph) => {
                mode = Some(Mode::Paragraph);
                buffer.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(Mode::Paragraph) = mode.take() {
                    doc.push(Block::Paragraph {
                        text: std::mem::take(&mut buffer),
                    });
                }
            }
            Event::Text(text) => buffer.push_str(&text),
            Event::Code(code) => buffer.push_str(&code),
            Event::SoftBreak | Event::HardBreak => buffer.push(' '),
            _ => {}
        }
    }

    doc
}

enum Mode {
    Heading(u8),
    Paragraph,
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

    #[test]
    fn parses_h1_and_paragraph() {
        let doc = parse("# Hello\n\nWorld");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Heading {
                    level: 1,
                    text: "Hello".into()
                },
                Block::Paragraph {
                    text: "World".into()
                },
            ]
        );
    }

    #[test]
    fn parses_all_heading_levels() {
        let md = "# One\n\n## Two\n\n### Three\n\n#### Four\n\n##### Five\n\n###### Six";
        let doc = parse(md);
        let levels: Vec<u8> = doc
            .blocks
            .iter()
            .filter_map(|b| match b {
                Block::Heading { level, .. } => Some(*level),
                _ => None,
            })
            .collect();
        assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn empty_input_yields_empty_document() {
        assert!(parse("").blocks.is_empty());
    }

    #[test]
    fn soft_break_becomes_space_in_paragraph() {
        let doc = parse("line one\nline two");
        assert_eq!(
            doc.blocks,
            vec![Block::Paragraph {
                text: "line one line two".into()
            }]
        );
    }
}
