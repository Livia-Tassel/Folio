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

/// A loaded reference template. Currently exposes only the raw `styles.xml`
/// content; later cycles will add `theme1.xml` and `numbering.xml`.
#[derive(Debug, Clone)]
pub struct Template {
    styles_xml: String,
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

        Ok(Self { styles_xml })
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
}

const BUILTIN_THEMES: &[(&str, &str)] = &[(
    "academic",
    include_str!("../themes/academic.styles.xml"),
)];

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
            let opts: SimpleFileOptions = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
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
}
