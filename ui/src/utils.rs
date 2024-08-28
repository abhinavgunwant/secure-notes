use std::path::Path;
use dirs_next::data_local_dir;

pub fn is_first_start() -> bool {
    match data_local_dir() {
        Some(mut path) => {
            path.push("secure-notes");

            if let Some(local_path) = path.to_str() {
                return !Path::new(local_path).exists();
            } else {
                return true
            }
        }

        None => false
    }
}

