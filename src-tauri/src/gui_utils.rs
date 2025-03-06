#[tauri::command(rename_all = "snake_case")]
pub fn select_file_from_explorer() -> String {
    return "Hi".to_owned();
}
