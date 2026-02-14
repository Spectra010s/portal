use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn set_config(key: &str, value: &str) -> Result<()> {
    // Load existing config if it exists
    let mut cfg = match PortalConfig::load_or_return().await? {
        Some(c) => c,
        None => PortalConfig::new_empty_for_set(key, value)?,
    };

    // Update the section
    let new_val = cfg.update_section(key, value)?;

    // Save back to disk
    cfg.save().await.context("Failed to save configuration")?;

    println!("Portal: Updated {} successfully as: {}", key, new_val);
    Ok(())
}
