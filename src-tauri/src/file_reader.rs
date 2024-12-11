use std::fs;

#[tauri::command]
pub fn get_video_files() -> Vec<String> {
    let mut video_files: Vec<String> = vec![];

    let folder_path: &str = "F:\\mov";
    //let dir: &Path = folder_path.as_ref();

    let files = fs::read_dir(folder_path).unwrap();

    for file in files {
        if let Ok(valid_file) = file {
            match valid_file.path().to_str() {
                Some(v) => video_files.push(v.to_owned()),
                None => {},
            }
        }
    }

    /*
    if dir.is_dir() {
        for entry in fs::read_dir(dir) {
            let entry = entry;
            let path: Path = entry.path();
            video_files.push(path.display().to_string());
        }
    }
    */

    video_files
}
