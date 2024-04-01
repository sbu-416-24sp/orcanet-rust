use config::{builder, Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Configurations {
    name: String,
    market: Vec<String>,
    files: Vec<String>,
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
            Ok(config_data) => {
              config_data
            }
            Err(_) => {
              return Self::default();
            }
          }
        }
        Err(e) => {
          return Self::default();
        }
      }
    }

    pub fn default() -> Self {
      let default = Configurations {
        name: "default".to_string(),
        market: vec!["localhost:50051".to_string()],
        files: vec![],
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

    pub fn get_files(&self) -> Vec<String> {
      self.files.clone()
    }

    pub fn add_market(&mut self, market: String) {
      if self.market.contains(&market) {
        return;
      }
      self.market.push(market);
      self.write();
    }

    pub fn add_file(&mut self, file: String) {
      // check if file is already in the list
      if self.files.contains(&file) {
        return;
      }
      self.files.push(file);
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
      if !self.files.contains(&file) {
        panic!("Path [{}] not found", file);
      }
      self.files.retain(|x| x != &file);
      self.write();
    }
}