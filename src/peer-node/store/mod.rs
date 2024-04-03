use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Configurations {
    name: String,
    market: Vec<String>,
    files: HashMap<String, i64>,
    // wallet: String, // not sure about implementation details, will revisit later
}

impl Configurations {
    pub fn new() -> Self {
        let config = Config::builder()
            // .set_default("market", "localhost:50051")?
            .add_source(File::new("config", FileFormat::Json))
            .build();

        match config {
            Ok(config) => {
                // lets try to deserialize the configuration
                let config_data = config.try_deserialize::<Configurations>();
                match config_data {
                    Ok(config_data) => config_data,
                    Err(_) => {
                        return Self::default();
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load configuration: {:?}", e);
                return Self::default();
            }
        }
    }

    pub fn default() -> Self {
        let default = Configurations {
            name: "default".to_string(),
            market: vec!["localhost:50051".to_string()],
            files: HashMap::new(),
        };
        default.write();
        return default;
    }

    pub fn write(&self) {
        // Serialize it to a JSON string.
        match serde_json::to_string(&self) {
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

    pub fn get_market(&self) -> Vec<String> {
        self.market.clone()
    }

    pub fn get_files(&self) -> HashMap<String, i64> {
        self.files.clone()
    }

    pub fn add_market(&mut self, market: String) {
        if self.market.contains(&market) {
            return;
        }
        self.market.push(market);
        self.write();
    }

    pub fn add_file(&mut self, file: String, price: i64) {
        self.files.insert(file, price);
        self.write();
    }

    pub fn remove_market(&mut self, market: String) {
        // if market is not in the list, panic
        if !self.market.contains(&market) {
            panic!("Market URL [{}] not found", market);
        }
        self.market.retain(|x| x != &market);
        self.write();
    }

    pub fn remove_file(&mut self, file: String) {
        // if file is not in the list, panic
        if !self.files.contains_key(&file) {
            panic!("File [{}] not found", file);
        }
        self.files.remove(&file);
        self.write();
    }
}
