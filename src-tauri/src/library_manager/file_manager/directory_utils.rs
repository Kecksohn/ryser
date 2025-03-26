use std::{fs, vec, io};
use std::fs::metadata;
use std::path::*;

// https://users.rust-lang.org/t/function-to-list-files-in-directories-and-in-subdirectories/46236/2
pub fn get_files_in_folder_and_subdirectories(path: &Path, subdirectories: &mut Vec<PathBuf>) -> io::Result<()> {
    if metadata(path)?.is_dir() {
        let paths = fs::read_dir(path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            if metadata(&full_path)?.is_dir() {
                get_files_in_folder_and_subdirectories(&full_path, subdirectories)?
            } else {
                subdirectories.push(full_path);
            }
        }
    }
    Ok(())
}