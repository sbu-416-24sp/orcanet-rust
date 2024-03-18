use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use glob::glob;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io;
use tokio::sync::RwLock;

pub struct FileMap {
    files: RwLock<HashMap<String, PathBuf>>,
}

pub type AsyncFileMap = Arc<FileMap>;

impl FileMap {
    pub fn new() -> Self {
        FileMap {
            files: RwLock::new(HashMap::new()),
        }
    }

    // Add all the files in a Unix-style glob to the map
    pub async fn add_all(&self, file_path: &str) -> Result<()> {
        // Get a write lock on the files map
        let mut files = self.files.write().await;
        for entry in glob(file_path)? {
            let path = entry?;
            let file = File::open(path.clone());
            match file {
                Ok(mut file) => {
                    let hash = self.hash_file(&mut file)?;
                    files.insert(hash, path);
                }
                Err(_) => {
                    println!("Failed to open file {:?}", path);
                }
            }
        }

        Ok(())
    }

    // Get a file path by its hash
    pub async fn get_file_path(&self, hash: &str) -> Option<PathBuf> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        let path = files.get(hash)?;
        Some(path.clone())
    }

    // Get a vector of all the hashes in the map
    pub async fn get_hashes(&self) -> Vec<String> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        files.keys().cloned().collect()
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
