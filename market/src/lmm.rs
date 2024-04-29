use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use proto::market::{FileInfo, User};
use serde::{Deserialize, Serialize};

pub(crate) const FILE_DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone)]
pub(crate) struct LocalMarketMap {
    inner: HashMap<FileInfoHash, LocalMarketEntry>,
    file_ttl: Duration,
}

impl LocalMarketMap {
    pub(crate) fn new(file_ttl: Duration) -> Self {
        if file_ttl.as_millis() == 0 {
            panic!("file_ttl cannot be zero");
        } else {
            Self {
                inner: HashMap::new(),
                file_ttl,
            }
        }
    }

    pub(crate) fn insert(&mut self, file_info_hash: FileInfoHash, supplier_info: SupplierInfo) {
        self.inner
            .insert(file_info_hash, (Instant::now(), supplier_info));
    }

    pub(crate) fn get_if_not_expired(
        &mut self,
        file_info_hash: &FileInfoHash,
    ) -> Option<SupplierInfo> {
        if let Some(entry) = self.inner.get(file_info_hash) {
            let elapsed_time = Instant::now().duration_since(entry.0);
            if elapsed_time >= self.file_ttl {
                self.inner.remove(file_info_hash);
                None
            } else {
                Some(entry.1.clone())
            }
        } else {
            None
        }
    }
}

impl Default for LocalMarketMap {
    fn default() -> Self {
        Self::new(FILE_DEFAULT_TTL)
    }
}

pub(crate) type LocalMarketEntry = (Instant, SupplierInfo);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct FileInfoHash(String);

impl FileInfoHash {
    #[inline(always)]
    pub const fn new(s: String) -> Self {
        Self(s)
    }

    #[inline(always)]
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.into()
    }
}

impl From<String> for FileInfoHash {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<FileInfoHash> for Vec<u8> {
    fn from(value: FileInfoHash) -> Self {
        value.0.into_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct SupplierInfo {
    pub(crate) file_info: FileInfo,
    pub(crate) user: User,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum FileResponse {
    HasFile(SupplierInfo),
    NoFile,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{net::Ipv4Addr, thread::sleep};

    #[test]
    fn test_insert_and_get_if_not_expired() {
        let mut lmm = LocalMarketMap::new(Duration::from_secs(1));
        let user = User {
            ip: Ipv4Addr::new(127, 0, 0, 1).to_string(),
            port: 8080,
            price: 100,
            name: "Alice".to_string(),
            id: "416".to_string(),
        };
        let file_info = FileInfo {
            file_hash: "foo".to_string(),
            chunk_hashes: vec!["1".into(), "2".into()],
            file_size: 8000,
            file_name: "a_file".to_string(),
        };
        let file_hash = FileInfoHash(file_info.hash_to_string());
        let supplier_info = SupplierInfo { file_info, user };
        lmm.insert(file_hash.clone(), supplier_info.clone());
        assert_eq!(lmm.get_if_not_expired(&file_hash), Some(supplier_info));
    }

    #[test]
    fn test_insert_and_should_expire() {
        let mut lmm = LocalMarketMap::new(Duration::from_millis(10));
        let file_info = FileInfo {
            file_hash: "foo".to_string(),
            chunk_hashes: vec!["1".into(), "2".into()],
            file_size: 8000,
            file_name: "a_file".to_string(),
        };
        let file_hash = FileInfoHash(file_info.hash_to_string());
        let user = User {
            ip: Ipv4Addr::new(127, 0, 0, 1).to_string(),
            port: 8080,
            price: 100,
            name: "Alice".to_string(),
            id: "416".to_string(),
        };
        let supplier_info = SupplierInfo { file_info, user };
        lmm.insert(file_hash.clone(), supplier_info);
        sleep(Duration::from_millis(20));
        assert_eq!(lmm.get_if_not_expired(&file_hash), None);
    }
}
