//! IPC commands exposed to the frontend.

/// Sanity-check command used by the frontend on startup.
#[tauri::command]
pub fn ping() -> &'static str {
    "pong"
}

/// Convert a Markdown file on disk into a .docx file.
///
/// If `output_path` is omitted, writes alongside the input with the
/// same stem and a `.docx` extension.
#[tauri::command]
pub fn convert_file(input_path: String, output_path: Option<String>) -> Result<String, String> {
    let input = std::path::PathBuf::from(&input_path);
    let output = match output_path {
        Some(p) => std::path::PathBuf::from(p),
        None => input.with_extension("docx"),
    };

    scribe_core::convert_file(&input, &output).map_err(|e| e.to_string())?;
    Ok(output.to_string_lossy().into_owned())
}

/// Convert a Markdown string in-memory; the frontend receives the
/// .docx bytes (base64) to save or preview as needed. Optionally
/// applies a built-in theme by name (see [`list_themes`]).
#[tauri::command]
pub fn convert_string(markdown: String, theme: Option<String>) -> Result<Vec<u8>, String> {
    let template = match theme.as_deref() {
        None | Some("") => None,
        Some(name) => Some(scribe_core::Template::builtin(name).map_err(|e| e.to_string())?),
    };
    scribe_core::convert_string_with_template(&markdown, template.as_ref())
        .map_err(|e| e.to_string())
}

/// Render a Markdown string as an HTML preview fragment (no <html> shell).
#[tauri::command]
pub fn preview_html(markdown: String) -> String {
    scribe_core::preview_html(&markdown)
}

/// Render a full standalone HTML document with embedded CSS — useful when
/// the frontend wants to drop the result straight into an iframe via srcdoc.
#[tauri::command]
pub fn preview_standalone(markdown: String) -> String {
    scribe_core::preview_standalone(&markdown)
}

/// Names of built-in themes the desktop UI can offer in its theme picker.
#[tauri::command]
pub fn list_themes() -> Vec<&'static str> {
    scribe_core::list_builtin_themes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong() {
        assert_eq!(ping(), "pong");
    }

    #[test]
    fn convert_string_produces_zip() {
        let bytes = convert_string("# Hi\n\nBody".to_string(), None).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn convert_string_with_theme_carries_theme_styles() {
        // Pass `theme=Some("academic")` and verify the output's styles.xml
        // carries the academic theme's signature (Times New Roman). This
        // proves the theme parameter actually reaches scribe-core, not
        // just that the IPC signature compiles.
        let bytes = convert_string("# Hi".to_string(), Some("academic".into())).unwrap();
        let cursor = std::io::Cursor::new(&bytes);
        let mut z = zip::ZipArchive::new(cursor).unwrap();
        let mut styles = String::new();
        use std::io::Read as _;
        z.by_name("word/styles.xml")
            .unwrap()
            .read_to_string(&mut styles)
            .unwrap();
        assert!(
            styles.contains("Times New Roman"),
            "expected academic theme; got: {styles}"
        );
    }

    #[test]
    fn convert_string_with_unknown_theme_returns_error() {
        let err = convert_string("# Hi".to_string(), Some("not-a-theme-xyz".into())).unwrap_err();
        assert!(err.contains("not-a-theme-xyz"), "got: {err}");
    }

    #[test]
    fn list_themes_returns_known_names() {
        let names = list_themes();
        assert!(names.contains(&"academic"));
        assert!(names.contains(&"thesis-cn"));
        assert!(names.contains(&"report"));
    }

    #[test]
    fn convert_file_round_trip() {
        let dir = tempdir();
        let input = dir.join("in.md");
        let output = dir.join("out.docx");
        std::fs::write(&input, "# A\n\nB").unwrap();

        let returned = convert_file(
            input.to_string_lossy().into_owned(),
            Some(output.to_string_lossy().into_owned()),
        )
        .unwrap();

        assert_eq!(returned, output.to_string_lossy());
        assert!(output.exists());
        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "scribe-tauri-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
