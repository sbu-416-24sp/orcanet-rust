use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use glob::glob;
use proto::market::{FileInfo, FileInfoHash};
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio::sync::RwLock;

#[allow(dead_code)]
pub struct FileMap {
    files: RwLock<HashMap<FileInfoHash, LocalFileInfo>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct LocalFileInfo {
    pub file_info: FileInfo,
    pub path: PathBuf,
    pub price: i64,
}

pub type AsyncFileMap = Arc<FileMap>;

pub async fn hash_file(file: &mut File) -> Result<FileHash> {
    // Get the hash of a file
    Ok(FileHash(
        generate_chunk_hashes(file)
            .await?
            .last()
            .ok_or(anyhow!("Empty file"))?
            .to_string(),
    ))
}
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct FileHash(String);

impl FileHash {
    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub async fn generate_chunk_hashes(file: &mut File) -> Result<Vec<String>> {
    let mut chunk_hashes = vec![];

    let mut sha256 = Sha256::new();
    let mut buffer = vec![0; FileAccessType::CHUNK_SIZE as usize];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        sha256.update(&buffer[..bytes_read]);
        let hash = format!("{:x}", sha256.clone().finalize());
        chunk_hashes.push(hash);
    }

    Ok(chunk_hashes)
}

pub async fn get_file_info(path: &PathBuf) -> Result<FileInfo> {
    let mut file = tokio::fs::File::open(path).await?;
    let file_hash = hash_file(&mut file).await?.0;
    let chunk_hashes = generate_chunk_hashes(&mut file).await?;
    let file_size = file.metadata().await?.len() as i64;
    let file_name = PathBuf::from(path)
        .file_name()
        .ok_or(anyhow::anyhow!("Failed to get file name"))?
        .to_str()
        .ok_or(anyhow!("Failed to convert file name to &str"))?
        .to_owned();
    Ok(FileInfo {
        file_hash,
        chunk_hashes,
        file_size,
        file_name,
    })
}

#[allow(dead_code)]
impl FileMap {
    pub fn default() -> Self {
        FileMap {
            files: RwLock::new(HashMap::new()),
        }
    }

    pub fn new(files: HashMap<FileInfoHash, LocalFileInfo>) -> Self {
        FileMap {
            files: RwLock::new(files),
        }
    }

    pub async fn set(&self, files: HashMap<FileInfoHash, LocalFileInfo>) -> Result<()> {
        let mut file_map = self.files.write().await;
        *file_map = files;
        Ok(())
    }

    pub async fn add_file(&self, file_path: &str, price: i64) -> Result<FileInfoHash> {
        // Get a write lock on the files map
        let mut files = self.files.write().await;

        let file_info = get_file_info(&PathBuf::from(file_path)).await?;
        let file_info_hash = file_info.get_hash();
        files.insert(
            file_info_hash.clone(),
            LocalFileInfo {
                file_info,
                path: PathBuf::from(file_path),
                price,
            },
        );

        Ok(file_info_hash)
    }

    // Add all the files in a Unix-style glob to the map
    pub async fn add_dir(&self, file_path: &str, price: i64) -> Result<()> {
        // Get a write lock on the files map
        let mut files = self.files.write().await;
        for entry in glob(file_path)? {
            let path = entry?;
            let file_info = get_file_info(&path).await?;
            let file_info_hash = file_info.get_hash();
            files.insert(
                file_info_hash.clone(),
                LocalFileInfo {
                    file_info,
                    path,
                    price,
                },
            );
        }
        Ok(())
    }

    pub async fn add_all(&self, files: HashMap<String, i64>) -> Result<()> {
        // Get a write lock on the files map
        for (file, price) in files {
            // check if this is a file or a directory
            match std::fs::metadata(&file) {
                Ok(metadata) => {
                    if metadata.is_file() {
                        self.add_file(&file, price).await?;
                    }
                    if metadata.is_dir() {
                        self.add_dir(&file, price).await?;
                    }
                }
                Err(_) => eprintln!("Failed to open file {file}"),
            }
        }
        Ok(())
    }

    // Get a file path by its hash
    pub async fn get_file_path(&self, hash: &FileInfoHash) -> Option<PathBuf> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        let path = files.get(hash)?.path.clone();
        Some(path)
    }

    // Get a vector of all the hashes in the map
    pub async fn get_hashes(&self) -> Vec<FileInfoHash> {
        // Get a read lock on the files map
        let files = self.files.read().await;

        files.keys().cloned().collect()
    }

    pub async fn get_prices(&self) -> HashMap<FileInfoHash, i64> {
        // Get a read lock on the files map
        let files = self.files.read().await;
        files
            .iter()
            .map(|(hash, file_info)| (hash.clone(), file_info.price))
            .collect()
    }
}

pub struct FileAccessType {
    file_path: String,
}

impl FileAccessType {
    // chunk size = 4mb
    const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

    pub fn new(file: &str) -> Result<Self> {
        Ok(FileAccessType {
            file_path: file.to_string(),
        })
    }

    pub async fn get_chunk(&self, desired_chunk: u64) -> Result<Option<Vec<u8>>> {
        // open the file
        let mut file = tokio::fs::File::open(&self.file_path).await?;

        // get total chunk number (file size / chunk size)
        let metadata = file.metadata().await?;
        let total_chunks = metadata.len() / Self::CHUNK_SIZE;

        // create a buffer to hold the file data
        let mut buffer = vec![0; Self::CHUNK_SIZE as usize];

        // check if the desired chunk is within the file size
        if desired_chunk > total_chunks {
            return Ok(None);
        }

        // seek to the desired chunk
        file.seek(SeekFrom::Start(desired_chunk * Self::CHUNK_SIZE))
            .await?;

        // read the chunk into the buffer
        let res = file.read_exact(&mut buffer).await;
        match res {
            Ok(_) => Ok(Some(buffer)),
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::UnexpectedEof {
                    // read until the end of the file
                    file.seek(SeekFrom::Start(desired_chunk * Self::CHUNK_SIZE))
                        .await?;
                    let mut buffer = vec![];
                    file.read_to_end(&mut buffer).await?;
                    Ok(Some(buffer))
                } else {
                    Err(e.into())
                }
            }
        }
    }
}
