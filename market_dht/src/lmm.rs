use std::{
    collections::HashMap,
    net::Ipv4Addr,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

pub(crate) const FILE_DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone)]
pub(crate) struct LocalMarketMap {
    inner: HashMap<FileHash, LocalMarketEntry>,
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

    pub(crate) fn insert(&mut self, file_hash: FileHash, supplier_info: SupplierInfo) {
        self.inner
            .insert(file_hash, (Instant::now(), supplier_info));
    }

    pub(crate) fn get_if_not_expired(&mut self, file_hash: &FileHash) -> Option<SupplierInfo> {
        if let Some(entry) = self.inner.get(file_hash) {
            let elapsed_time = Instant::now().duration_since(entry.0);
            if elapsed_time >= self.file_ttl {
                self.inner.remove(file_hash);
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
pub(crate) struct FileHash(pub(crate) Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SupplierInfo {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub price: i64,
    pub username: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::thread::sleep;

    #[test]
    fn test_insert_and_get_if_not_expired() {
        let mut lmm = LocalMarketMap::new(Duration::from_secs(1));
        let file_hash = FileHash(vec![1]);
        let supplier_info = SupplierInfo {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 8080,
            price: 100,
            username: "Alice".to_string(),
        };
        lmm.insert(file_hash.clone(), supplier_info.clone());
        sleep(Duration::from_millis(150));
        assert_eq!(lmm.get_if_not_expired(&file_hash), Some(supplier_info));
    }

    #[test]
    fn test_insert_and_should_expire() {
        let mut lmm = LocalMarketMap::new(Duration::from_millis(250));
        let file_hash = FileHash(vec![1]);
        let supplier_info = SupplierInfo {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 8080,
            price: 100,
            username: "Alice".to_string(),
        };
        lmm.insert(file_hash.clone(), supplier_info);
        sleep(Duration::from_millis(300));
        assert_eq!(lmm.get_if_not_expired(&file_hash), None);
    }
}
