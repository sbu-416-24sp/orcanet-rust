use std::collections::HashMap;

use orcanet_market::{bridge::spawn, Config, Peer};

use proto::market::{FileInfo, FileInfoHash, HoldersResponse, User};

use anyhow::{anyhow, Result};
use anyhow::{anyhow, Result};

// test market
#[cfg(feature = "test_local_market")]
pub struct MarketClient {
    local: HashMap<FileInfoHash, HoldersResponse>,
}
#[cfg(feature = "test_local_market")]
impl MarketClient {
    pub async fn new(config: Config) -> Result<Self> {
        Ok(MarketClient {
            local: Default::default(),
        })
    }
    pub async fn check_holders(&mut self, file_info_hash: FileInfoHash) -> Result<HoldersResponse> {
        match self.local.get(&file_info_hash) {
            Some(res) => Ok(res.clone()),
            None => Err(anyhow!("not found!")),
        }
    }
    pub async fn register_file(
        &mut self,
        user: User,
        file_info_hash: FileInfoHash,
        file_info: FileInfo,
    ) -> Result<()> {
        match self.local.get_mut(&file_info_hash) {
            Some(res) => {
                res.holders.retain(|u| u != &user);
                res.holders.push(user);
                Ok(())
            }
            None => {
                self.local.insert(
                    file_info_hash,
                    HoldersResponse {
                        file_info: Some(file_info),
                        holders: vec![user],
                    },
                );
                Ok(())
            }
        }
    }
}

#[cfg(not(feature = "test_local_market"))]
pub struct MarketClient {
    inner: Peer,
}

#[cfg(not(feature = "test_local_market"))]
impl MarketClient {
    // TODO: DO NOT PR MERGE FOR PEER NODE UNLESS WE PROVIDE A CONFIG HERE
    //
    // Initialize a new MarketClient, connecting to the given market service address
    pub async fn new(config: Config) -> Result<Self> {
        let peer = spawn(config)?;

        Ok(MarketClient { inner: peer })
    }

    // Get a list of producers for a given file hash
    pub async fn check_holders(&mut self, file_info_hash: FileInfoHash) -> Result<HoldersResponse> {
        return Ok(HoldersResponse {
            file_info: Some(FileInfo {
                file_hash: "x".into(),
                chunk_hashes: vec!["y".into()],
                file_size: 3,
                file_name: "z".into(),
            }),
            holders: vec![User {
                id: "a".into(),
                name: "user".into(),
                ip: "0.0.0.0".into(),
                port: 80,
                price: 9999,
            }],
        });
        todo!()
    }

    // Register a new producer
    pub async fn register_file(
        &mut self,
        user: User,
        file_info_hash: FileInfoHash,
        file_info: FileInfo,
    ) -> Result<()> {
        todo!()
    }
}
