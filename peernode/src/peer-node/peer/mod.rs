use orcanet_market::{bridge::spawn, Config, Peer};

use proto::market::{FileInfo, HoldersResponse, User};

use anyhow::{anyhow, Result};

use std::collections::HashMap;

// test market
#[cfg(feature = "test_local_market")]
pub struct MarketClient {
    local: HashMap<String, HoldersResponse>,
    //// update in later commit
    //local: HashMap<FileInfoHash, HoldersResponse>,
}
#[cfg(feature = "test_local_market")]
impl MarketClient {
    pub async fn new(_config: Config) -> Result<Self> {
        Ok(MarketClient {
            local: Default::default(),
        })
    }
    pub async fn check_holders(&mut self, file_info_hash: String) -> Result<HoldersResponse> {
        //// update in later commit
        //pub async fn check_holders(&mut self, file_info_hash: FileInfoHash) -> Result<HoldersResponse> {
        match self.local.get(&file_info_hash) {
            Some(res) => Ok(res.clone()),
            None => Err(anyhow!("not found!")),
        }
    }

    pub async fn register_file(
        &mut self,
        id: String,
        name: String,
        ip: String,
        port: i32,
        price: i64,
        file_hash: String,
    ) -> Result<()> {
        //// update in later commit
        //pub async fn register_file(
        //    &mut self,
        //    user: User,
        //    file_info_hash: FileInfoHash,
        //    file_info: FileInfo,
        //) -> Result<()> {
        let user = User {
            id,
            name,
            ip,
            port,
            price,
        };
        match self.local.get_mut(&file_hash) {
            Some(res) => {
                res.holders.retain(|u| u.id != user.id);
                res.holders.push(user.clone());
                Ok(())
            }
            None => {
                self.local.insert(
                    file_hash.clone(),
                    HoldersResponse {
                        file_info: Some(FileInfo {
                            file_hash,
                            chunk_hashes: vec![],
                            file_size: 0,
                            file_name: "foo".into(),
                        }),
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
    pub async fn check_holders(&mut self, file_hash: String) -> Result<HoldersResponse> {
        todo!()
        // println!("gRPC: Checking holders for file hash {}", file_hash);
        // let request = CheckHoldersRequest { file_hash };

        // let response = self.client.check_holders(request).await?.into_inner();

        // Ok(response)
    }

    // Register a new producer
    pub async fn register_file(
        &mut self,
        id: String,
        name: String,
        ip: String,
        port: i32,
        price: i64,
        file_hash: String,
    ) -> Result<()> {
        todo!()
        // let user = User {
        //     id,
        //     name,
        //     ip,
        //     port,
        //     price,
        // };
        // let file = RegisterFileRequest {
        //     user: Some(user),
        //     file_hash: file_hash.clone(),
        // };

        // self.client.register_file(file).await?;
        // println!("gRPC: Registered producer for file hash {}", file_hash);

        // Ok(())
    }
}
