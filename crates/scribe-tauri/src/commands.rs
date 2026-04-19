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
/// .docx bytes (base64) to save or preview as needed.
#[tauri::command]
pub fn convert_string(markdown: String) -> Result<Vec<u8>, String> {
    scribe_core::convert_string(&markdown).map_err(|e| e.to_string())
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
        let bytes = convert_string("# Hi\n\nBody".to_string()).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
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
