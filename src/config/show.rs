use {
    crate::config::models::PortalConfig,
    anyhow::{Context, Result},
    tracing::{debug, trace},
};

pub async fn show_config_value(key: &str) -> Result<()> {
    // load the file
    trace!("Attempting to retrieve config value for key: {}", key);
    let cfg = PortalConfig::load_all()
        .await
        .context("Failed to load configuration")?;

    // match and show the value for the key
    let val = cfg.get_key_value(key)?;
    debug!("Retrieved value for {}: {}", key, val);

    println!("{}: {}", key, val);
    Ok(())
}
