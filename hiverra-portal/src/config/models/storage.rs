use std::path::PathBuf;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageConfig {
   pub download_dir: PathBuf
 }


impl StorageConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        match field {
            "download_dir" => {
                self.download_dir = PathBuf::from(value);
                Ok(self.download_dir.display().to_string())
            }
            _ => Err(anyhow!("Unknown field '{}' in [storage]", field)),
        }
    }
    
    pub fn get_value(&self, field: &str) -> Result<String> {
            match field {
                "download_dir" => Ok(self.download_dir.display().to_string()),
                _ => Err(anyhow!("Unknown field '{}' in [storage]", field)),
        }
    }

    
}

