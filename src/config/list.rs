use {
    crate::config::models::PortalConfig,
    anyhow::{Context, Result},
    tracing::{debug, trace},
};

pub async fn list_config() -> Result<String> {
    // load config
    trace!("Listing all configuration keys");
    let cfg = PortalConfig::load_all().await?;

    let output = toml::to_string_pretty(&cfg).context("Failed to format config")?;
    trace!("Raw TOML configuration length: {} bytes", output.len());

    // replace toml = and quotes
    let clean_list = output.replace(" = ", ": ").replace("\"", "");
    debug!("Cleaned config list prepared for display");

    println!("\nCurrent Portal Configuration: \n \n{}", clean_list);
    Ok(clean_list)
}
