use anyhow::{anyhow, Error};

use directories::ProjectDirs;
use std::fs;

use super::{Library, VideoElement};

pub(super) fn get_library(identifier: &str) -> Result<Library, Error> {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        let library_json_filepath = proj_dir
            .data_local_dir()
            .join(identifier)
            .join("library.json");

        match fs::File::open(&library_json_filepath) {
            Ok(json_file) => match serde_json::from_reader(json_file) {
                Ok(library) => Ok(library),
                Err(error) => Err(anyhow!(
                    "Error extracting {} , error: {}",
                    library_json_filepath.to_str().unwrap(),
                    error
                )),
            },
            Err(error) => Err(anyhow!(
                "Problem opening ".to_owned()
                    + library_json_filepath.to_str().unwrap()
                    + &error.to_string(),
            )),
        }
    } else {
        Err(anyhow!("Could not get project config dir paths"))
    }
}

pub(super) fn write_library(library: &Library) {
    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        // Create library folder if it does not exist
        let library_folder = proj_dir.data_local_dir().join(&library.id);
        if !library_folder.exists() {
            match fs::create_dir_all(&library_folder) {
                Ok(()) => {}
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Write json to library.json
        let library_json_file = library_folder.join("library.json");
        let file = fs::File::create(&library_json_file);
        match file {
            Ok(file) => {
                let _ = serde_json::to_writer_pretty(file, &library);
                println!("Wrote to {}", &library_json_file.to_str().unwrap());
            }
            Err(error) => panic!("Problem creating config.json: {error:?}"),
        }
    }
}
