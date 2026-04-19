//! scribe-core: orchestration layer used by both the GUI and the CLI.
//!
//! Holds the public conversion entry points. The GUI in `scribe-tauri`
//! and the `scribe-cli` binary call these — they never touch the
//! lower-level parser/emitter crates directly.

use std::fs;
use std::path::Path;

pub mod error;

pub use error::{ConvertError, Result};

/// Convert a Markdown string to `.docx` bytes.
pub fn convert_string(markdown: &str) -> Result<Vec<u8>> {
    let doc = scribe_parser::parse(markdown);
    scribe_docx::emit(&doc).map_err(ConvertError::Emit)
}

/// Convert a Markdown file at `input` into a `.docx` file at `output`.
pub fn convert_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<()> {
    let markdown = fs::read_to_string(input.as_ref()).map_err(ConvertError::Read)?;
    let bytes = convert_string(&markdown)?;
    fs::write(output.as_ref(), bytes).map_err(ConvertError::Write)?;
    Ok(())
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
