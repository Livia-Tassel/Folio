//! scribe-core: orchestration layer used by both the GUI and the CLI.
//!
//! Holds the public conversion entry points. The GUI in `scribe-tauri`
//! and the `scribe-cli` binary call these — they never touch the
//! lower-level parser/emitter crates directly.

use std::fs;
use std::path::Path;

pub mod error;

pub use error::{ConvertError, Result};
pub use scribe_template::{list_builtin_themes, Template};

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
    convert_file_with_template(input, output, None)
}

/// Convert a Markdown string to `.docx` bytes, optionally honouring a
/// reference [`Template`] for styles.
pub fn convert_string_with_template(
    markdown: &str,
    template: Option<&Template>,
) -> Result<Vec<u8>> {
    let doc = scribe_parser::parse(markdown);
    scribe_docx::emit_with_options(
        &doc,
        scribe_docx::EmitOptions {
            base_dir: None,
            styles_xml_override: template.map(|t| t.styles_xml()),
        },
    )
    .map_err(ConvertError::Emit)
}

/// Convert a Markdown file to a `.docx` file, optionally honouring a
/// reference [`Template`] for styles.
pub fn convert_file_with_template(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    template: Option<&Template>,
) -> Result<()> {
    let input_path = input.as_ref();
    let markdown = fs::read_to_string(input_path).map_err(ConvertError::Read)?;
    let doc = scribe_parser::parse(&markdown);
    let base = input_path.parent().map(|p| p.to_path_buf());
    let bytes = scribe_docx::emit_with_options(
        &doc,
        scribe_docx::EmitOptions {
            base_dir: base,
            styles_xml_override: template.map(|t| t.styles_xml()),
        },
    )
    .map_err(ConvertError::Emit)?;
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

    #[test]
    fn convert_string_with_template_replaces_styles() {
        // Build a fake reference doc whose styles.xml has a unique sentinel,
        // then convert any markdown using it. The output must carry that
        // sentinel — proof the template was honoured end-to-end.
        let sentinel = "FolioReferenceTemplateSentinel_123";
        let custom = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:styleId="{sentinel}"><w:name w:val="{sentinel}"/></w:style>
</w:styles>"#
        );

        // Pack the styles.xml into a minimal docx-like zip.
        let mut docx_bytes = Vec::new();
        {
            use std::io::Write as _;
            let cursor = std::io::Cursor::new(&mut docx_bytes);
            let mut zip = zip::ZipWriter::new(cursor);
            zip.start_file(
                "word/styles.xml",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(custom.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        let template = Template::from_reference_doc_bytes(&docx_bytes).unwrap();
        let out = convert_string_with_template("# Hi", Some(&template)).unwrap();

        // Extract word/styles.xml from the output and confirm it carries
        // the sentinel.
        let cursor = std::io::Cursor::new(&out);
        let mut z = zip::ZipArchive::new(cursor).unwrap();
        let mut buf = String::new();
        use std::io::Read as _;
        z.by_name("word/styles.xml")
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        assert!(
            buf.contains(sentinel),
            "expected output styles.xml to carry sentinel; got: {buf}"
        );
    }
}
