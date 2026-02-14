use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn set_config(key: &str, value: &str) -> Result<()> {
    // Load existing config
    let mut cfg = PortalConfig::load().await?;

    // call update_section fn
    let new_val = cfg.update_section(key, value)?;

    //  Save back to disk
    cfg.save().await.context("Failed to save configuration")?;

    println!("Portal: Updated {} successfully as: {}", key, new_val);
    Ok(())
}
