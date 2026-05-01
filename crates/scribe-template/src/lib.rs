//! scribe-template: load reference.docx templates and extract their styles.
//!
//! A reference doc is just a `.docx` archive with the page setup, fonts,
//! and named styles a user wants. We pull `word/styles.xml` out of it and
//! hand it back as text — `scribe-docx` will splice that into the output
//! archive in place of Folio's built-in styles.
//!
//! This crate has no opinion on what *should* be in the styles XML; it
//! is a load-and-extract layer. Validation is a separate concern.

use std::io::{Cursor, Read};
use std::path::Path;

const STYLES_PATH: &str = "word/styles.xml";
const DOCUMENT_PATH: &str = "word/document.xml";

/// A loaded reference template. Currently exposes only the raw `styles.xml`
/// content; later cycles will add `theme1.xml` and `numbering.xml`.
#[derive(Debug, Clone)]
pub struct Template {
    styles_xml: String,
    section_xml: Option<String>,
}

impl Template {
    /// Load a template from a `.docx` archive's raw bytes.
    pub fn from_reference_doc_bytes(bytes: &[u8]) -> Result<Self, TemplateError> {
        let cursor = Cursor::new(bytes);
        let mut zip = zip::ZipArchive::new(cursor).map_err(TemplateError::Zip)?;

        let mut entry = zip.by_name(STYLES_PATH).map_err(|e| match e {
            zip::result::ZipError::FileNotFound => TemplateError::MissingStyles,
            other => TemplateError::Zip(other),
        })?;

        let mut buf = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut buf).map_err(TemplateError::Read)?;
        let styles_xml = String::from_utf8(buf).map_err(TemplateError::Utf8)?;
        drop(entry);

        // word/document.xml is optional — a styles-only fragment is still
        // a valid reference.
        let section_xml = match zip.by_name(DOCUMENT_PATH) {
            Ok(mut doc_entry) => {
                let mut doc_buf = Vec::with_capacity(doc_entry.size() as usize);
                doc_entry
                    .read_to_end(&mut doc_buf)
                    .map_err(TemplateError::Read)?;
                let doc_xml = String::from_utf8(doc_buf).map_err(TemplateError::Utf8)?;
                extract_section_pr(&doc_xml)
            }
            Err(zip::result::ZipError::FileNotFound) => None,
            Err(other) => return Err(TemplateError::Zip(other)),
        };

        Ok(Self {
            styles_xml,
            section_xml,
        })
    }

    /// Load a template from a `.docx` file on disk.
    pub fn from_reference_doc(path: impl AsRef<Path>) -> Result<Self, TemplateError> {
        let bytes = std::fs::read(path).map_err(TemplateError::Read)?;
        Self::from_reference_doc_bytes(&bytes)
    }

    /// Construct a [`Template`] directly from a `word/styles.xml` string.
    /// Useful when the styles are produced by something other than a real
    /// `.docx` (e.g. a built-in theme baked into the binary).
    pub fn from_styles_xml(xml: impl Into<String>) -> Self {
        Self {
            styles_xml: xml.into(),
            section_xml: None,
        }
    }

    /// Load a [`Template`] from a built-in theme name. See
    /// [`list_builtin_themes`] for the supported names.
    pub fn builtin(name: &str) -> Result<Self, TemplateError> {
        for (theme_name, xml) in BUILTIN_THEMES {
            if *theme_name == name {
                return Ok(Self::from_styles_xml(*xml));
            }
        }
        Err(TemplateError::UnknownBuiltin(name.to_string()))
    }

    /// Raw `word/styles.xml` content as it appears in the reference doc.
    pub fn styles_xml(&self) -> &str {
        &self.styles_xml
    }

    /// Page setup (`<w:sectPr>...</w:sectPr>`) lifted from the reference
    /// doc's `word/document.xml`. `None` when the reference has no document
    /// part or no sectPr inside it (built-in themes never carry one).
    pub fn section_xml(&self) -> Option<&str> {
        self.section_xml.as_deref()
    }
}

/// Find the last `<w:sectPr ...>...</w:sectPr>` element in `document.xml`.
/// In OOXML the body's terminating sectPr describes the page setup of the
/// (final) section, which is what we want to inherit. A document can also
/// have per-paragraph sectPr inside `<w:pPr>` for section breaks; we treat
/// the last one as the canonical "page setup" the user authored.
fn extract_section_pr(document_xml: &str) -> Option<String> {
    let close_tag = "</w:sectPr>";
    let close_idx = document_xml.rfind(close_tag)?;
    // Search backwards for the matching opening tag. Accept both
    // `<w:sectPr>` and `<w:sectPr ...>` (with attributes).
    let head = &document_xml[..close_idx];
    let open_idx = head.rfind("<w:sectPr")?;
    let end_idx = close_idx + close_tag.len();
    Some(document_xml[open_idx..end_idx].to_string())
}

const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("academic", include_str!("../themes/academic.styles.xml")),
    ("thesis-cn", include_str!("../themes/thesis-cn.styles.xml")),
];

/// Names of themes that [`Template::builtin`] understands.
pub fn list_builtin_themes() -> Vec<&'static str> {
    BUILTIN_THEMES.iter().map(|(name, _)| *name).collect()
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("failed to read reference doc: {0}")]
    Read(#[source] std::io::Error),

    #[error("reference doc is not a valid zip archive: {0}")]
    Zip(#[source] zip::result::ZipError),

    #[error("reference doc contains no word/styles.xml")]
    MissingStyles,

    #[error("word/styles.xml is not valid UTF-8: {0}")]
    Utf8(#[source] std::string::FromUtf8Error),

    #[error("unknown built-in theme: {0}")]
    UnknownBuiltin(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn build_minimal_docx(styles_xml: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts: SimpleFileOptions =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("word/styles.xml", opts).unwrap();
            zip.write_all(styles_xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn loads_styles_xml_from_reference_doc_bytes() {
        let want = r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:styleId="MyHeading"/></w:styles>"#;
        let docx = build_minimal_docx(want);

        let template = Template::from_reference_doc_bytes(&docx).unwrap();

        assert_eq!(template.styles_xml(), want);
    }

    #[test]
    fn archive_without_word_styles_returns_missing_styles_error() {
        // Build a zip with some other entry but no word/styles.xml.
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts: SimpleFileOptions = SimpleFileOptions::default();
            zip.start_file("word/document.xml", opts).unwrap();
            zip.write_all(b"<w:document/>").unwrap();
            zip.finish().unwrap();
        }

        let err = Template::from_reference_doc_bytes(&buf).unwrap_err();

        assert!(
            matches!(err, TemplateError::MissingStyles),
            "expected MissingStyles, got {err:?}"
        );
    }

    #[test]
    fn non_zip_bytes_return_zip_error() {
        let err = Template::from_reference_doc_bytes(b"definitely not a zip").unwrap_err();
        assert!(matches!(err, TemplateError::Zip(_)), "got {err:?}");
    }

    #[test]
    fn loads_from_path_on_disk() {
        let want = r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:styleId="OnDisk"/></w:styles>"#;
        let docx = build_minimal_docx(want);

        let dir = std::env::temp_dir().join("scribe-template-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("ref-{}.docx", std::process::id()));
        std::fs::write(&path, &docx).unwrap();

        let template = Template::from_reference_doc(&path).unwrap();

        assert_eq!(template.styles_xml(), want);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_path_returns_read_error() {
        let path = std::env::temp_dir().join("scribe-template-nonexistent-xyz.docx");
        let _ = std::fs::remove_file(&path); // make sure it's gone

        let err = Template::from_reference_doc(&path).unwrap_err();

        assert!(matches!(err, TemplateError::Read(_)), "got {err:?}");
    }

    #[test]
    fn from_styles_xml_constructs_template_directly() {
        let xml = "<w:styles/>";
        let t = Template::from_styles_xml(xml);
        assert_eq!(t.styles_xml(), xml);
    }

    #[test]
    fn builtin_academic_theme_loads_and_carries_times_new_roman_default() {
        // The "academic" theme is shipped inside the binary; users get
        // it via `--theme academic` without supplying any file. The
        // contract: it loads, and its body font is Times New Roman.
        let t = Template::builtin("academic").unwrap();
        let xml = t.styles_xml();
        assert!(
            xml.contains("Times New Roman"),
            "academic theme styles should reference Times New Roman; got: {xml}"
        );
    }

    #[test]
    fn unknown_builtin_theme_returns_unknown_builtin_error() {
        let err = Template::builtin("definitely-not-a-real-theme-xyz").unwrap_err();
        assert!(
            matches!(&err, TemplateError::UnknownBuiltin(name) if name == "definitely-not-a-real-theme-xyz"),
            "got {err:?}"
        );
    }

    #[test]
    fn list_builtin_themes_returns_known_names() {
        let names = list_builtin_themes();
        assert!(
            names.contains(&"academic"),
            "expected 'academic' in list_builtin_themes(); got {names:?}"
        );
    }

    #[test]
    fn builtin_thesis_cn_theme_loads_and_uses_chinese_fonts() {
        // The "thesis-cn" theme is the Chinese-language counterpart to
        // "academic": 宋体 body, 黑体 headings, 1.5 line height, 2-char
        // first-line indent. The contract: it loads, and its eastAsia
        // run fonts reference 宋体 (SimSun) somewhere.
        let t = Template::builtin("thesis-cn").unwrap();
        let xml = t.styles_xml();
        assert!(
            xml.contains("宋体") || xml.contains("SimSun"),
            "thesis-cn theme styles should reference 宋体 / SimSun; got: {xml}"
        );
        assert!(
            xml.contains("黑体") || xml.contains("SimHei"),
            "thesis-cn theme styles should reference 黑体 / SimHei for headings; got: {xml}"
        );
    }

    #[test]
    fn list_builtin_themes_includes_thesis_cn() {
        assert!(list_builtin_themes().contains(&"thesis-cn"));
    }

    #[test]
    fn extracts_section_pr_from_reference_doc_document_xml() {
        // Pandoc-style page-setup inheritance: sectPr lives inside the
        // body of word/document.xml; we lift it out so emit can swap it
        // into the output. Build a minimal docx whose document.xml has
        // a sectPr with a recognizable margin value, then verify
        // Template::section_xml() returns that same sectPr.
        let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>body</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="9999" w:h="9999"/><w:pgMar w:top="7777" w:right="0" w:bottom="0" w:left="0" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr>
  </w:body>
</w:document>"#;
        let styles_xml =
            r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"/>"#;

        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(cursor);
            let opts: SimpleFileOptions = SimpleFileOptions::default();
            zip.start_file("word/styles.xml", opts).unwrap();
            zip.write_all(styles_xml.as_bytes()).unwrap();
            zip.start_file("word/document.xml", opts).unwrap();
            zip.write_all(document_xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }

        let template = Template::from_reference_doc_bytes(&buf).unwrap();

        let section = template
            .section_xml()
            .expect("expected sectPr to be extracted");
        assert!(
            section.contains(r#"w:top="7777""#),
            "expected our sentinel margin in extracted sectPr; got: {section}"
        );
        // The extracted snippet should be a complete <w:sectPr>...</w:sectPr> element.
        assert!(section.starts_with("<w:sectPr"), "got: {section}");
        assert!(section.ends_with("</w:sectPr>"), "got: {section}");
    }

    #[test]
    fn section_xml_is_none_when_reference_has_no_sect_pr() {
        // Some minimal docs (or ours from earlier tests) have no sectPr.
        // Template::section_xml() must report that cleanly rather than
        // panicking or returning a malformed snippet.
        let bytes = build_minimal_docx("<w:styles/>"); // no document.xml at all
        let t = Template::from_reference_doc_bytes(&bytes).unwrap();
        assert_eq!(t.section_xml(), None);
    }
}
