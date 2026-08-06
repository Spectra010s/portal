use {
    crate::config::models::PortalConfig,
    anyhow::{Context, Result},
    tracing::{debug, info, trace},
};

pub async fn set_config(key: &str, value: &str) -> Result<()> {
    // Load existing config if it exists
    trace!("Attempting to resolve configuration for update: {}", key);
    let mut cfg = match PortalConfig::load_or_return().await? {
        Some(c) => {
            trace!("Existing configuration found, modifying field");
            c
        }
        None => {
            debug!(
                "No existing configuration found; initializing new one for key: {}",
                key
            );
            PortalConfig::new_empty_for_set(key, value)?
        }
    };

    // Update the section
    trace!("Updating configuration key-value pair: {} = {}", key, value);
    let new_val = cfg.update_section(key, value)?;

    // Save back to disk
    debug!("Saving updated configuration to disk");
    cfg.save().await.context("Failed to save configuration")?;

    println!("Portal: Updated {} successfully as: {}", key, new_val);
    info!("Configuration field '{}' updated to '{}'", key, new_val);
    Ok(())
}
