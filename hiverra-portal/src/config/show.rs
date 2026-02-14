use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn show_config_value(key: &str) -> Result<()> {
    // load the file
    let cfg = PortalConfig::load()
        .await
        .context("Failed to load configuration")?;
    // match and show the value for the key
    let new_val = cfg.get_key_value(key)?;
    
    
    println!("{}: {}", key, new_val);
    Ok(())
}



