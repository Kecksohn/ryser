use std::fs;

#[tauri::command]
pub fn get_video_files() -> String {
    let mut video_files: Vec<String> = vec![];

    let folder_path: &str = "F:/mov";
    //let dir: &Path = folder_path.as_ref();

    let paths = fs::read_dir(folder_path).unwrap();

    for path in paths {
        if let Ok(path) = path {
            let path_display = path.path();
            match path_display.to_str() {
                Some(v) => video_files.push(v.to_owned()),
                None => video_files.push(String::from(" a ")),
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

    video_files[0].clone()
}
