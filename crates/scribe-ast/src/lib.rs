//! scribe-ast: typed Markdown AST used by all Scribe stages.
//!
//! Minimal v0: only the block kinds produced by M1-5 (H1–H6 headings and
//! paragraphs). M2 expands this to the full set specified in
//! `docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md` §3.3.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading { level: u8, text: String },
    Paragraph { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_accumulates_blocks() {
        let mut doc = Document::new();
        doc.push(Block::Heading {
            level: 1,
            text: "Hello".into(),
        });
        doc.push(Block::Paragraph {
            text: "World".into(),
        });
        assert_eq!(doc.blocks.len(), 2);
    }
}
