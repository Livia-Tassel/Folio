//! Scribe desktop app — library entry. `main.rs` is the thin binary shim.

pub mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::convert_file,
            commands::convert_string,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
