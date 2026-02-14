 use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
 pub struct NetworkConfig {
    pub default_port: u16
   }
    
impl NetworkConfig {

pub fn update(&mut self, field: &str, value: &str) -> Result<String> {

    match field {
            "default_port" => {
                self.default_port = value
                    .parse()
                    .context("Invalid Port number: Port must be a number")?;
                Ok(self.default_port.to_string())
                }
            _ => return Err(anyhow!("Unknown field in [network]: {}", field)),
        }    
  }
  pub fn get_value(&self, field: &str) -> Result<String> {
            match field {
                "default_port" => Ok(self.default_port.to_string()),
                _ => Err(anyhow!("Unknown field '{}' in [network]", field)),
            }
  }
  }