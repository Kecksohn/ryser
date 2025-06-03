use super::library_manager::{LIBRARIES, library};

pub(crate) fn get_all_library_ids() -> Vec<String> {
    let mut library_ids: Vec<String> = vec![];
    for library in LIBRARIES.lock().unwrap().iter() {
        library_ids.push(library.id.clone());
    }
    
    library_ids
}

pub(crate) fn get_all_library_names() -> Vec<String> {
    let mut library_names: Vec<String> = vec![];
    for library in LIBRARIES.lock().unwrap().iter() {
        library_names.push(library.id.clone());
    }
    
    library_names
}