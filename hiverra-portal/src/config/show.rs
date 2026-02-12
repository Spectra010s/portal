use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn show_config_value(key: &str) -> Result<()> {
    // load the file
    let cfg = PortalConfig::load()
        .await
        .context("Failed to load configuration")?;
    // match and show the value for the key
    match key.to_lowercase().as_str() {
        "port" => println!("Current Default port: {}", cfg.default_port.to_string()),
        "username" => println!("Current username: {}", cfg.username),
        _ => println!("Key '{}' not recognized", key),
    }
    Ok(())
}
