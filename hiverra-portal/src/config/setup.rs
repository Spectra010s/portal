use {crate::config::models::PortalConfig, anyhow::Result, inquire::Confirm};

pub async fn handle_setup() -> Result<()> {
    let path = PortalConfig::get_dir().await?.join("config.toml");

    if path.exists() {
        println!("⚠ Configuration already exists at {:?}", path);
        let ans = Confirm::new("Do you want to overwrite it?")
            .with_default(false)
            .prompt()?;

        if ans {
            println!("Proceeding with setup...");
        } else {
            println!("Setup cancelled. Use 'portal config set' to change specific values.");
            return Ok(());
        }
    }

    // Run interactive_init
    PortalConfig::interactive_init().await?;
    Ok(())
}
