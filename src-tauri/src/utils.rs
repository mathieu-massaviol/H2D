/// Opens a native OS "Save As" dialog and writes the given content to the
/// chosen file. Returns the saved path, or `None` if the user cancelled.
#[tauri::command]
pub async fn save_csv(content: String, default_filename: String) -> Result<Option<String>, String> {
    let path = tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_file_name(&default_filename)
            .add_filter("CSV", &["csv"])
            .save_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    match path {
        Some(p) => {
            std::fs::write(&p, content.as_bytes())
                .map_err(|e| format!("Failed to write file: {e}"))?;
            Ok(Some(p.to_string_lossy().to_string()))
        }
        None => Ok(None),
    }
}
