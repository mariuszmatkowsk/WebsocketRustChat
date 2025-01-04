use std::collections::HashMap;
use std::fs::{read, read_dir};
use std::path::Path;

#[derive(Clone)]
pub struct FileStorage {
    files: HashMap<String, Vec<u8>>,
}

impl FileStorage {
    pub fn new(path: &Path) -> Option<Self> {
        let mut files = HashMap::new();
        let entries = match read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("Can't read provided dir path: {:?}, error: {}", path, err);
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
                        "Can't read content of file: {:?}, error: {}",
                        file_entry.path(),
                        err
                    );
                    continue;
                }
            };

            files.insert(file_entry.file_name().into_string().unwrap(), file_content);
            println!(
                "Successfully loaded file: {}.",
                file_entry.file_name().to_str().unwrap()
            );
        }

        Some(Self { files })
    }

    pub fn get(&self, file: &str) -> Option<&Vec<u8>> {
        self.files.get(file)
    }
}
