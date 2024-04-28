use crate::{
    consumer::encode::EncodedUser,
    peer::MarketClient,
    producer::{
        self,
        files::{get_file_info, FileHash, FileMap, LocalFileInfo},
        jobs::Jobs,
    },
};
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use config::{Config, File, FileFormat};
use orcanet_market::{BootNodes, Multiaddr};
use proto::market::{FileInfoHash, User};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive()]
pub struct Configurations {
    // this is the struct that will be used to store the configurations
    props: Properties,
    http_client: Option<tokio::task::JoinHandle<()>>,
    market_client: Option<MarketClient>,
    // and shared state apparently
    // {Peer Id -> Peer Info}
    discovered_peers: HashMap<String, PeerInfo>,
    jobs: Jobs,
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    // must be a separate serializable struct so can read from config.json file
    name: String,
    market: String,
    files: HashMap<FileInfoHash, LocalFileInfo>,
    tokens: HashMap<EncodedUser, String>,
    port: String,
    // market config
    boot_nodes: Option<BootNodes>,
    public_address: Option<Multiaddr>,
    // wallet: String, // not sure about implementation details, will revisit later
    theme: Theme,
}

// ok whatever just add it
#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    dark,
    light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub user: User,
}

// TODO: Put prices and path attached to the same hash in config file, and then construct the hashmaps from that
impl Configurations {
    pub async fn new() -> Self {
        let config = Config::builder()
            .add_source(File::new("config", FileFormat::Json))
            .build();
        let props = match config {
            Ok(config) => {
                // lets try to deserialize the configuration
                let config_data = config.try_deserialize::<Properties>();
                match config_data {
                    Ok(config_data) => config_data,
                    Err(_) => {
                        return Self::default().await;
                    }
                }
            }
            Err(_) => {
                return Self::default().await;
            }
        };
        Configurations {
            props,
            http_client: None,
            market_client: None,
            jobs: Jobs::new(),
            discovered_peers: HashMap::new(),
        }
    }

    pub async fn default() -> Self {
        // this is the default configuration
        let default = Configurations {
            props: Properties {
                name: "default".to_string(),
                market: "localhost:50051".to_string(),
                files: HashMap::new(),
                tokens: HashMap::new(),
                port: "8080".to_string(),
                boot_nodes: None,
                public_address: None,
                theme: Theme::dark,
            },
            http_client: None,
            market_client: None,
            jobs: Jobs::new(),
            discovered_peers: HashMap::new(),
        };
        default.write();
        default
    }

    // write to config.json
    pub fn write(&self) {
        // Serialize it to a JSON string.
        match serde_json::to_string(&self.props) {
            Ok(default_config_json) => {
                // Write the string to the file.
                match std::fs::write("config.json", default_config_json) {
                    Ok(_) => {
                        return;
                    }
                    Err(_) => {
                        eprintln!("Failed to write to file");
                    }
                }
            }
            Err(_) => {
                eprintln!("Failed to serialize configuration");
            }
        }
    }

    pub async fn get_hash(&self, file_path: String) -> Result<FileInfoHash> {
        Ok(producer::files::get_file_info(&PathBuf::from(file_path))
            .await?
            .get_hash())
    }

    pub fn jobs(&self) -> &Jobs {
        &self.jobs
    }

    pub fn jobs_mut(&mut self) -> &mut Jobs {
        &mut self.jobs
    }

    pub fn get_files(&self) -> HashMap<FileInfoHash, LocalFileInfo> {
        self.props.files.clone()
    }

    pub fn get_prices(&self) -> HashMap<FileInfoHash, i64> {
        self.props
            .files
            .iter()
            .map(|(hash, file_info)| (hash.clone(), file_info.price))
            .collect()
    }

    pub fn get_port(&self) -> String {
        self.props.port.clone()
    }

    pub fn get_boot_nodes(&self) -> Option<BootNodes> {
        self.props.boot_nodes.clone()
    }

    pub fn get_public_address(&self) -> Option<Multiaddr> {
        self.props.public_address.clone()
    }

    pub fn get_theme(&self) -> Theme {
        self.props.theme
    }

    pub fn get_token(&mut self, producer_id: EncodedUser) -> String {
        match self.props.tokens.get(&producer_id).cloned() {
            Some(token) => token,
            None => {
                let token = "token".to_string();
                self.set_token(producer_id, token.clone());
                self.write();
                token
            }
        }
    }

    pub fn set_token(&mut self, producer_id: EncodedUser, token: String) {
        self.props.tokens.insert(producer_id, token);
        self.write();
    }

    pub fn set_port(&mut self, port: String) {
        self.props.port = port;
        self.write();
    }

    pub fn set_boot_nodes(&mut self, boot_nodes: Option<BootNodes>) {
        self.props.boot_nodes = boot_nodes;
        self.write();
    }

    pub fn set_public_address(&mut self, public_address: Option<Multiaddr>) {
        self.props.public_address = public_address;
        self.write();
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.props.theme = theme;
        self.write();
    }

    // add every file in the directory to the list
    #[async_recursion]
    pub async fn add_dir(&mut self, file_path: &Path, price: i64) -> Result<usize> {
        // assume that the file_path is a directory
        let mut num_added = 0;
        for entry in fs::read_dir(file_path)? {
            let path = &entry?.path();
            // check if this is a file or a directory
            if path.is_dir() {
                num_added += self.add_dir(path, price).await?;
            }
            if path.is_file() {
                self.add_file(path, price).await?;
                num_added += 1;
            }
        }
        Ok(num_added)
    }

    // add a single file to the list
    pub async fn add_file(&mut self, file_path: &PathBuf, price: i64) -> Result<FileInfoHash> {
        let file_info = get_file_info(file_path).await?;
        let file_info_hash = file_info.get_hash();
        self.props.files.insert(
            file_info_hash.clone(),
            LocalFileInfo {
                file_info,
                path: file_path.to_owned(),
                price,
            },
        );

        Ok(file_info_hash)
    }

    // cli command to add a file/dir to the list
    pub async fn add_file_path(&mut self, file_path: &PathBuf, price: i64) -> Result<usize> {
        // check if this is a file or a directory
        let mut num_added = 0;
        match std::fs::metadata(file_path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    self.add_file(file_path, price).await?;
                    num_added += 1;
                }
                if metadata.is_dir() {
                    num_added += self.add_dir(file_path, price).await?
                }
            }
            Err(e) => Err(anyhow!("Failed to open file {file_path:?}: {e}"))?,
        }
        self.write();
        Ok(num_added)
    }

    pub async fn remove_file(&mut self, file_path: String) -> Result<()> {
        // get the hash of the file
        let file_info = get_file_info(&PathBuf::from(file_path.clone())).await?;
        let hash = file_info.get_hash();

        if !self.props.files.contains_key(&hash) {
            panic!("File [{}] not found", file_path);
        }
        self.props.files.remove(&hash);
        self.write();
        Ok(())
    }

    pub fn set_http_client(&mut self, http_client: tokio::task::JoinHandle<()>) {
        self.http_client = Some(http_client);
    }

    pub fn is_http_running(&self) -> bool {
        // git blame this
        if self.http_client.is_some() {
            return true;
        }
        return false;
    }

    pub async fn start_http_client(&mut self, port: String) {
        // stop the current http client
        if let Some(http_client) = self.http_client.take() {
            match producer::stop_server(http_client).await {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to stop HTTP server");
                }
            }
        }

        // Set the port
        self.set_port(port.clone());

        let join = // must run in separate thread so does not block cli inputs
            producer::start_server(self.props.files.clone(), port).await;
        self.set_http_client(join);
    }

    pub async fn stop_http_client(&mut self) {
        if let Some(http_client) = self.http_client.take() {
            match producer::stop_server(http_client).await {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to stop HTTP server");
                }
            }
        }
    }

    pub async fn get_market_client(&mut self) -> Result<&mut MarketClient> {
        if self.market_client.is_none() {
            let mut config = orcanet_market::Config::builder();
            if let Some(boot_nodes) = self.props.boot_nodes.clone() {
                if !boot_nodes.is_empty() {
                    config = config.set_boot_nodes(boot_nodes);
                }
            }
            if let Some(public_address) = self.props.public_address.clone() {
                config = config.set_public_address(public_address.clone());
            }
            let market_client = MarketClient::new(config.build()).await?;
            self.market_client = Some(market_client);
        }
        let market_client = self.market_client.as_mut().unwrap(); // safe to unwrap because we just set it
        Ok(market_client)
    }

    pub fn get_peer(&self, peer_id: &str) -> Option<&PeerInfo> {
        self.discovered_peers.get(peer_id)
    }
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.discovered_peers.values().cloned().collect()
    }
    pub fn remove_peer(&mut self, peer_id: &str) -> Option<PeerInfo> {
        self.discovered_peers.remove(peer_id)
    }
}
