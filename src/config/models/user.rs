use {
    anyhow::{Result, anyhow},
    serde::{Deserialize, Serialize},
    tracing::{debug, trace},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
    pub username: Option<String>,
}

impl UserConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        trace!(
            "UserConfig: update field '{}' with value '{}'",
            field, value
        );
        match field {
            "username" => {
                self.username = if value.ends_with("@portal") {
                    trace!("Value already contains '@portal' suffix");
                    Some(value.to_string())
                } else {
                    let suffixed = format!("{}@portal", value);
                    trace!("Appended '@portal' suffix: {}", suffixed);
                    Some(suffixed)
                };
                debug!(
                    "Username updated in config: {:?}",
                    self.username.as_ref().unwrap()
                );
                self.username
                    .clone()
                    .ok_or_else(|| anyhow!("username not set"))
            }
            _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
        }
    }

    pub fn get_value(&self, field: &str) -> Result<String> {
        trace!("UserConfig: get_value for field '{}'", field);
        match field {
            "username" => {
                let val = self
                    .username
                    .clone()
                    .ok_or_else(|| anyhow!("username not set"))?;
                debug!("Retrieved username from config: {}", val);
                Ok(val)
            }
            _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
        }
    }
}
