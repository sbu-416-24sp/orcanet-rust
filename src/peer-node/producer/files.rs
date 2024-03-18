use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use glob::glob;
use sha2::Sha256;
use sha2::Digest;
use tokio::sync::Mutex;
use std::fs::File;
use anyhow::Result;
use std::io;

pub struct FileMap {
    files: HashMap<String, PathBuf>,
}

pub type AsyncFileMap = Arc<Mutex<FileMap>>;

impl FileMap {
    pub fn new() -> Self {
        FileMap {
            files: HashMap::new(),
        }
    }

    // Add all the files in a Unix-style glob to the map
    pub fn add_all(&mut self, file_path: &str) -> Result<()> {
        for entry in glob(file_path)? {
            let path = entry?;
            let file = File::open(path.clone());
            match file {
                Ok(mut file) => {
                    let hash = self.hash_file(&mut file)?;
                    self.files.insert(hash, path);
                }
                Err(_) => {
                    println!("Failed to open file {:?}", path);
                }
            }
        }

        Ok(())
    }

    // Get a file path by its hash
    pub fn get_file_path(&self, hash: &str) -> Option<PathBuf> {
        let path = self.files.get(hash)?;
        Some(path.clone())
    }

    // Get a vector of all the hashes in the map
    pub fn get_hashes(&self) -> Vec<String> {
        self.files.keys().cloned().collect()
    }

    // Get the hash of a file
    fn hash_file(&self, file: &mut File) -> Result<String> {
        let mut sha256 = Sha256::new();
        io::copy(file, &mut sha256)?;
        let hash = sha256.finalize();
        let hash = format!("{:x}", hash);
        Ok(hash)
    }
}