use orcanet_market::{bridge::spawn, Config, Peer, SuccessfulResponse};

use proto::market::{FileInfo, FileInfoHash, HoldersResponse, User};

use anyhow::{anyhow, Result};
use std::collections::HashMap;

// test market
#[cfg(feature = "test_local_market")]
#[derive(Debug)]
pub struct MarketClient {
    local: HashMap<FileInfoHash, HoldersResponse>,
}
#[cfg(feature = "test_local_market")]
impl MarketClient {
    pub async fn new(_config: Config) -> Result<Self> {
        Ok(MarketClient {
            local: Default::default(),
        })
    }
    pub async fn check_holders(&self, file_info_hash: FileInfoHash) -> Result<HoldersResponse> {
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
#[derive(Debug)]
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
    pub async fn check_holders(&self, file_info_hash: FileInfoHash) -> Result<HoldersResponse> {
        match self.inner.check_holders(file_info_hash).await {
            Ok(SuccessfulResponse::CheckHolders(res)) => Ok(res),
            Ok(_) => unreachable!(),
            Err(e) => Err(anyhow!("{e}")),
        }
    }

    // Register a new producer
    pub async fn register_file(
        &mut self,
        user: User,
        file_info_hash: FileInfoHash,
        file_info: FileInfo,
    ) -> Result<()> {
        match self.inner.register_file(user, file_info_hash, file_info).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("{e}")),
        }
    }
}
