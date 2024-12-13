use std::fs;

#[tauri::command(rename_all = "snake_case")]
pub fn get_video_files(folder_path: &str) -> Vec<String> {
    let mut video_files: Vec<String> = vec![];

    let files = fs::read_dir(folder_path).unwrap();
    for file in files {
        if let Ok(valid_file) = file {
            match valid_file.path().to_str() {
                Some(v) => video_files.push(v.to_owned()),
                None => {},
            }
        }
    }

    video_files
}
