use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct UserConfig{
   pub username: String
}


impl UserConfig {
    pub fn update(&mut self, field: &str, value: &str) -> Result<String> {
        match field {
            "username" => {
                self.username = if value.ends_with("@portal") {
                    value.to_string()
                } else {
                    format!("{}@portal", value)
                };
                Ok(self.username.clone())
            }
            _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
        }
    }
    
    pub fn get_value(&self, field: &str) -> Result<String> {
    match field {
                "username" => Ok(self.username.clone()),
                _ => Err(anyhow!("Unknown field '{}' in [user]", field)),
            }
     }
}
