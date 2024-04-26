use orcanet_market::{bridge::spawn, Peer, SupplierInfo};

use anyhow::Result;

pub struct MarketClient {
    inner: Peer,
}

impl MarketClient {
    // TODO: DO NOT PR MERGE FOR PEER NODE UNLESS WE PROVIDE A CONFIG HERE
    //
    // Initialize a new MarketClient, connecting to the given market service address
    pub async fn new(market: String) -> Result<Self> {
        let peer = spawn(Default::default())?;

        Ok(MarketClient { inner: peer })
    }

    // Get a list of producers for a given file hash
    pub async fn check_holders(&mut self, file_hash: String) -> Result<Vec<SupplierInfo>> {
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
