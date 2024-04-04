use crate::grpc::MarketClient;
use crate::producer;
use anyhow::Result;
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

#[derive()]
pub struct Configurations {
    props: Properties,
    http_client: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Serialize, Deserialize)]
pub struct Properties {
    name: String,
    market: String,
    files: HashMap<String, PathBuf>,
    prices: HashMap<String, i64>,
    port: String,
    // wallet: String, // not sure about implementation details, will revisit later
}

impl Configurations {
    pub async fn new() -> Self {
        let config = Config::builder()
            // .set_default("market", "localhost:50051")?
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
            Err(e) => {
                eprintln!("Failed to load configuration: {:?}", e);
                return Self::default().await;
            }
        };
        Configurations {
            props,
            http_client: None,
        }
    }

    pub async fn default() -> Self {
        let default = Configurations {
            props: Properties {
                name: "default".to_string(),
                market: "localhost:50051".to_string(),
                files: HashMap::new(),
                prices: HashMap::new(),
                port: "8080".to_string(),
            },
            http_client: None,
        };
        default.write();
        return default;
    }

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

    pub fn get_hash(&self, file_path: String) -> Result<String> {
        // open the file
        let mut file = std::fs::File::open(file_path)?;
        // hash the file
        let hash = producer::files::hash_file(&mut file)?;
        Ok(hash)
    }

    pub fn get_market(&self) -> String {
        self.props.market.clone()
    }

    pub fn get_files(&self) -> HashMap<String, PathBuf> {
        self.props.files.clone()
    }

    pub fn get_prices(&self) -> HashMap<String, i64> {
        self.props.prices.clone()
    }

    pub fn get_port(&self) -> String {
        self.props.port.clone()
    }

    pub fn set_port(&mut self, port: String) {
        self.props.port = port;
        self.write();
    }

    pub fn add_file(&mut self, file: String, price: i64) {
        // hash the file
        let hash = match self.get_hash(file.clone()) {
            Ok(hash) => hash,
            Err(_) => {
                panic!("Failed to hash file");
            }
        };

        self.props.files.insert(hash.clone(), PathBuf::from(file));
        self.props.prices.insert(hash, price);
        self.write();
    }

    pub fn remove_file(&mut self, file_path: String) {
        // get the hash of the file
        let hash = match self.get_hash(file_path.clone()) {
            Ok(hash) => hash,
            Err(_) => {
                panic!("Failed to hash file");
            }
        };

        // if file is not in the list, panic
        if !self.props.files.contains_key(&hash) || !self.props.prices.contains_key(&hash) {
            panic!("File [{}] not found", file_path);
        }
        self.props.files.remove(&hash);
        self.props.prices.remove(&hash);
        self.write();
    }

    pub fn set_http_client(&mut self, http_client: tokio::task::JoinHandle<()>) {
        self.http_client = Some(http_client);
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

        // set the port
        self.set_port(port.clone());

        let join = producer::start_server(self.props.files.clone(), self.props.prices.clone(), port).await;
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

}
