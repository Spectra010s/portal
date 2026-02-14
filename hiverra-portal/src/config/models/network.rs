use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkConfig {
    pub default_port: Option<u16>,
}

impl NetworkConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        match field {
            "default_port" => {
                let port = value
                    .parse::<u16>()
                    .context("Invalid Port number: Port must be a number")?;
                self.default_port = Some(port);
                Ok(port.to_string())
            }
            _ => Err(anyhow!("Unknown field in [network]: {}", field)),
        }
    }

    pub fn get_value(&self, field: &str) -> Result<String> {
        match field {
            "default_port" => match self.default_port {
                Some(port) => Ok(port.to_string()),
                None => Ok(String::new()),
            },
            _ => Err(anyhow!("Unknown field '{}' in [network]", field)),
        }
    }
}
