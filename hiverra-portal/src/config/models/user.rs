use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig {
    pub username: Option<String>,
}

impl UserConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        match field {
            "username" => {
                self.username = if value.ends_with("@portal") {
                    Some(value.to_string())
                } else {
                    format!("{}@portal", value).into()
                };
                self.username
                    .clone()
                    .ok_or_else(|| anyhow!("username not set"))
            }
            _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
        }
    }

    pub fn get_value(&self, field: &str) -> Result<String> {
        match field {
            "username" => self
                .username
                .clone()
                .ok_or_else(|| anyhow!("username not set")),
            _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
        }
    }
}
