//! Smoke integration test: full MD → DOCX pipeline round-trip.

use std::io::Read;

#[test]
fn converts_hello_world_md_to_docx() {
    let markdown = "# Hello\n\nWorld";
    let bytes = scribe_core::convert_string(markdown).expect("convert_string should succeed");

    // .docx is a zip — magic bytes check.
    assert!(
        bytes.len() > 30,
        "docx body too small: {} bytes",
        bytes.len()
    );
    assert_eq!(&bytes[0..2], b"PK", "output is not a zip archive");

    // Unzip and pull document.xml, then assert the text made it in.
    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("valid zip");

    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .expect("document.xml should be present")
        .read_to_string(&mut xml)
        .expect("document.xml should be UTF-8");

    assert!(
        xml.contains("Hello"),
        "heading text missing in document.xml"
    );
    assert!(
        xml.contains("World"),
        "paragraph text missing in document.xml"
    );
    assert!(
        xml.contains("Heading1"),
        "Heading1 style reference missing — template styling won't apply"
    );
}

#[test]
fn all_heading_levels_get_their_style() {
    let md = "# L1\n\n## L2\n\n### L3\n\n#### L4\n\n##### L5\n\n###### L6";
    let bytes = scribe_core::convert_string(md).unwrap();

    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();

    for level in 1..=6 {
        let style = format!("Heading{level}");
        assert!(
            xml.contains(&style),
            "{style} style should be applied to level {level} heading"
        );
    }
}
