use {
    anyhow::{Context, Result, anyhow},
    serde::{Deserialize, Serialize},
    tracing::{debug, trace},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkConfig {
    pub default_port: Option<u16>,
}

impl NetworkConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        trace!(
            "NetworkConfig: update field '{}' with value '{}'",
            field, value
        );
        match field {
            "default_port" => {
                let port = value
                    .parse::<u16>()
                    .context("Invalid Port number: Port must be a number")?;
                self.default_port = Some(port);
                debug!("Default port updated in config: {}", port);
                Ok(port.to_string())
            }
            _ => Err(anyhow!("Unknown field in [network]: {}", field)),
        }
    }
    pub fn get_value(&self, field: &str) -> Result<String> {
        trace!("NetworkConfig: get_value for field '{}'", field);
        match field {
            "default_port" => {
                let p = self
                    .default_port
                    .map(|p| p.to_string())
                    .ok_or_else(|| anyhow!("default_port not set"))?;
                debug!("Retrieved default_port from config: {}", p);
                Ok(p)
            }
            _ => Err(anyhow!("Unknown field '{}' in [network]", field)),
        }
    }
}
