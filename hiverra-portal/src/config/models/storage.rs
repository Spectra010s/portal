use {
    anyhow::{Result, anyhow},
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct StorageConfig {
    pub download_dir: Option<PathBuf>,
}

impl StorageConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        match field {
            "download_dir" => {
                let path = PathBuf::from(value);
                self.download_dir = Some(path.clone());
                Ok(path.display().to_string())
            }
            _ => Err(anyhow!("Unknown field '{}' in [storage]", field)),
        }
    }
    pub fn get_value(&self, field: &str) -> Result<String> {
        match field {
            "download_dir" => self
                .download_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .ok_or_else(|| anyhow!("download_dir not set")),
            _ => Err(anyhow!("Unknown field '{}' in [storage]", field)),
        }
    }
}
