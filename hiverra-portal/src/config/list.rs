use crate::config::models::PortalConfig;
use anyhow::{Context, Result};

pub async fn list_config() -> Result<String> {
    // load config
    let cfg = PortalConfig::load_all().await?;

    let output = toml::to_string_pretty(&cfg).context("Failed to format config")?;

    // replace toml = and quotes
    let clean_list = output.replace(" = ", ": ").replace("\"", "");

    println!("\nCurrent Portal Configuration: \n \n{}", clean_list);
    Ok(clean_list)
}
