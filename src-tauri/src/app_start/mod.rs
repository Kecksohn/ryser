use std::fs;
use std::fs::File;
use default_config_json::get_default_config_as_json;
use directories::ProjectDirs;

mod default_config_json;

    
pub(crate) fn read_config() {

    if let Some(proj_dir) = ProjectDirs::from("", "", "ryser") {
        
        // Create config dir if it doesn't exist
        let config_dir = proj_dir.config_local_dir();
        if! config_dir.exists() {
            match fs::create_dir_all(config_dir) {
                Ok(()) => {},
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Create data dir if it doesn't exist
        let data_dir = proj_dir.data_local_dir();
        if! data_dir.exists() {
            match fs::create_dir_all(data_dir) {
                Ok(()) => {},
                Err(error) => panic!("Problem creating folder: {error:?}"),
            }
        }

        // Create config.json if it doesn't exist
        let config_filepath = config_dir.join("config.json");
        if! config_filepath.exists() {
            let file = File::create(config_filepath);
            match file {
                Ok(file) => {
                    let _ = serde_json::to_writer_pretty(file, &get_default_config_as_json());
                    println!("Wrote to config.json");
                },
                Err(error) => panic!("Problem creating config.json: {error:?}"),
            }
        }
    }
}