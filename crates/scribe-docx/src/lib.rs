//! scribe-docx: emit `.docx` bytes from a [`scribe_ast::Document`].
//!
//! Minimal v0: headings (Heading1..Heading6 styles) and paragraphs.
//! M2 expands to the full feature set (inline formatting, lists,
//! tables, code blocks, footnotes, images, math, cross-refs).

use docx_rs::{Docx, Paragraph, Run};
use scribe_ast::{Block, Document};

/// Convert a [`Document`] into `.docx` bytes.
///
/// Uses Word's built-in `Heading1`–`Heading6` paragraph styles so the
/// output picks up the target template's heading styling automatically.
pub fn emit(doc: &Document) -> anyhow::Result<Vec<u8>> {
    let mut out = Docx::new();

    for block in &doc.blocks {
        let para = match block {
            Block::Heading { level, text } => {
                let style_id = heading_style_id(*level);
                Paragraph::new()
                    .style(&style_id)
                    .add_run(Run::new().add_text(text))
            }
            Block::Paragraph { text } => Paragraph::new().add_run(Run::new().add_text(text)),
        };
        out = out.add_paragraph(para);
    }

    let mut buf: Vec<u8> = Vec::new();
    out.build().pack(&mut std::io::Cursor::new(&mut buf))?;
    Ok(buf)
}

fn heading_style_id(level: u8) -> String {
    let clamped = level.clamp(1, 6);
    format!("Heading{clamped}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_valid_zip_container() {
        let mut doc = Document::new();
        doc.push(Block::Heading {
            level: 1,
            text: "Hello".into(),
        });
        doc.push(Block::Paragraph {
            text: "World".into(),
        });

        let bytes = emit(&doc).unwrap();
        assert!(bytes.len() > 4, "output must not be empty");
        assert_eq!(&bytes[0..2], b"PK", "not a zip container");
    }

    #[test]
    fn heading_style_id_clamps() {
        assert_eq!(heading_style_id(0), "Heading1");
        assert_eq!(heading_style_id(3), "Heading3");
        assert_eq!(heading_style_id(9), "Heading6");
    }
}
