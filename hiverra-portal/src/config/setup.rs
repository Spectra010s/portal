use {
    crate::config::models::PortalConfig,
    anyhow::Result,
    inquire::Confirm,
    tracing::{debug, trace},
};

pub async fn handle_setup() -> Result<()> {
    trace!("Determining configuration directory for setup check");
    let path = PortalConfig::get_dir().await?.join("config.toml");

    if path.exists() {
        trace!("Found existing configuration file at {:?}", path);
        println!("⚠ Configuration already exists at {:?}", path);
        let ans = Confirm::new("Do you want to overwrite it?")
            .with_default(false)
            .prompt()?;

        if ans {
            debug!("User confirmed overwrite of existing config at {:?}", path);
            println!("Proceeding with setup...");
        } else {
            debug!("User cancelled configuration overwrite.");
            println!("Setup cancelled. Use 'portal config set' to change specific values.");
            return Ok(());
        }
    } else {
        trace!(
            "No existing configuration found at {:?}. Starting fresh setup.",
            path
        );
    }

    // Run interactive_init
    PortalConfig::interactive_init().await?;
    Ok(())
}
