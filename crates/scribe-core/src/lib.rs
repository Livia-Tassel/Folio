//! scribe-core: orchestration layer used by both the GUI and the CLI.
//!
//! Holds the public conversion entry points. The GUI in `scribe-tauri`
//! and the `scribe-cli` binary call these — they never touch the
//! lower-level parser/emitter crates directly.

use std::fs;
use std::path::Path;

pub mod error;

pub use error::{ConvertError, Result};

/// Convert a Markdown string to `.docx` bytes. Relative image paths can't
/// be resolved in string mode; use [`convert_file`] for that.
pub fn convert_string(markdown: &str) -> Result<Vec<u8>> {
    let doc = scribe_parser::parse(markdown);
    scribe_docx::emit(&doc).map_err(ConvertError::Emit)
}

/// Convert a Markdown file at `input` into a `.docx` file at `output`.
///
/// Relative image paths inside the Markdown are resolved relative to
/// the input file's parent directory.
pub fn convert_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<()> {
    let input_path = input.as_ref();
    let markdown = fs::read_to_string(input_path).map_err(ConvertError::Read)?;
    let doc = scribe_parser::parse(&markdown);
    let base = input_path.parent().map(|p| p.to_path_buf());
    let bytes = scribe_docx::emit_with_base(&doc, base).map_err(ConvertError::Emit)?;
    fs::write(output.as_ref(), bytes).map_err(ConvertError::Write)?;
    Ok(())
}

/// Render a Markdown string into an HTML preview fragment for the live-preview pane.
pub fn preview_html(markdown: &str) -> String {
    let doc = scribe_parser::parse(markdown);
    scribe_preview::render(&doc)
}

/// Render a Markdown string into a complete standalone HTML document.
pub fn preview_standalone(markdown: &str) -> String {
    let doc = scribe_parser::parse(markdown);
    scribe_preview::render_standalone(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_string_returns_docx_bytes() {
        let bytes = convert_string("# Hi\n\nBody").unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }
}
