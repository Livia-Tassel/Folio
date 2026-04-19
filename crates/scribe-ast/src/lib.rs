//! scribe-ast: typed Markdown AST used by all Folio stages.
//!
//! The AST has two layers: [`Block`] (document-level nodes like headings,
//! paragraphs, lists, tables) and [`Inline`] (text runs, emphasis, links,
//! inline code). Block nodes hold `Vec<Inline>` for their content.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph {
        content: Vec<Inline>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    /// A code block (fenced ``` or indented). `lang` is the info string
    /// after the fence (e.g. "rust"), empty for plain code.
    CodeBlock {
        lang: String,
        code: String,
    },
    /// Ordered (with start number) or unordered list.
    List {
        ordered: bool,
        /// Start number for ordered lists; ignored for unordered.
        start: u64,
        items: Vec<ListItem>,
    },
    /// GFM-style table. First row is the header.
    Table {
        alignments: Vec<Alignment>,
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    ThematicBreak,
    /// Display-math equation (produced by `$$...$$` or `\[...\]`).
    MathBlock {
        latex: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// `Some(checked)` for GFM task list items, `None` for regular items.
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    /// Bold (`**foo**` / `__foo__`).
    Strong(Vec<Inline>),
    /// Italic (`*foo*` / `_foo_`).
    Emphasis(Vec<Inline>),
    /// Strikethrough (`~~foo~~`).
    Strikethrough(Vec<Inline>),
    /// Inline code (`` `foo` ``).
    Code(String),
    /// Hyperlink. `title` is the optional tooltip.
    Link {
        url: String,
        title: String,
        content: Vec<Inline>,
    },
    /// Footnote reference — `[^label]`. The definition is looked up in
    /// [`Document::footnotes`] at emit time.
    FootnoteRef(String),
    /// Inline math equation (`$...$` or `\(...\)`).
    InlineMath(String),
    /// Inline image reference. Loaded and sized by the emitter.
    Image {
        url: String,
        alt: String,
        title: String,
    },
    /// Hard line break (two trailing spaces or `\` at end of line).
    HardBreak,
    /// Soft line break rendered as a space.
    SoftBreak,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
    /// Footnote definitions, keyed by their Markdown label (without the `^`).
    /// Populated during parsing; consumed during emission.
    pub footnotes: std::collections::BTreeMap<String, Vec<Block>>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn add_footnote(&mut self, label: String, blocks: Vec<Block>) {
        self.footnotes.insert(label, blocks);
    }
}

/// Helpers for building inlines in tests and simple parsers.
pub mod build {
    use super::Inline;

    pub fn text(s: impl Into<String>) -> Inline {
        Inline::Text(s.into())
    }

    pub fn strong(inlines: Vec<Inline>) -> Inline {
        Inline::Strong(inlines)
    }

    pub fn emph(inlines: Vec<Inline>) -> Inline {
        Inline::Emphasis(inlines)
    }

    pub fn code(s: impl Into<String>) -> Inline {
        Inline::Code(s.into())
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
            content: vec![Inline::Text("Hi".into())],
        });
        doc.push(Block::Paragraph {
            content: vec![Inline::Text("Body".into())],
        });
        assert_eq!(doc.blocks.len(), 2);
    }

    #[test]
    fn inline_tree_is_cloneable_and_comparable() {
        let a = Inline::Strong(vec![Inline::Text("x".into())]);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
