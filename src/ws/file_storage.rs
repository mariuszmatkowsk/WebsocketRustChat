use std::collections::HashMap;
use std::fs::{read, read_dir};
use std::path::Path;

pub struct FileStorage {
    files: HashMap<String, Vec<u8>>,
}

impl FileStorage {
    pub fn new(path: &Path) -> Option<Self> {
        let mut files = HashMap::new();
        let entries = match read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("Culd not read provided dir path: {:?}, error: {}", path, err);
                return None;
            }
        };

        for file_entry in entries {
            let file_entry = match file_entry {
                Ok(file_entry) => file_entry,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    continue;
                }
            };

            let file_content = match read(file_entry.path()) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "Could not read content file: {:?}, error: {}",
                        file_entry.path(),
                        err
                    );
                    continue;
                }
            };

            files.insert(file_entry.file_name().into_string().unwrap(), file_content);
        }

        Some(Self { files })
    }
}
