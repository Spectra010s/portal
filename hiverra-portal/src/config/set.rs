use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn set_config(key: &str, value: &str) -> Result<()> {
    // 1. Load existing config
    let mut cfg = PortalConfig::load().await?;

    // 2. Update the specific key
    let new_val = cfg.update_field(key, value)?;

    // 3. Save back to disk
    cfg.save().await.context("Failed to save configuration")?;

    println!("Portal: Succesfully saved {} as: {}", key, new_val);

    Ok(())
}
