  use {
    crate::config::models::PortalConfig, 
    anyhow::{Context, Result, anyhow},
    home::home_dir,
    inquire::Text,
    std::path::PathBuf,
    tokio::fs::create_dir_all
};

// Use CLI-provided path, or config, or prompt user if neither exists
 pub async fn get_target_dir(dir: &Option<PathBuf>) -> Result<PathBuf> {
 let target_dir = if let Some(dir) = dir {
        dir.clone()
    } else if let Some(cfg) = PortalConfig::load_or_return().await? {
        if let Some(d) = &cfg.storage.download_dir {
            println!("Portal: Using directory from config: {}", d.display());
            d.clone()
        } else {
            println!("Portal: Config exists but download directory not set.");
            let default_path = home_dir()
                .ok_or_else(|| anyhow!("Could not find home directory"))?
                .join("Downloads")
                .display()
                .to_string();

            let dir_string = Text::new("Portal: Where should Portal save this file?")
                .with_default(&default_path)
                .with_help_message("Enter a valid folder path.")
                .prompt()
                .context("No directory provided")?;

            PathBuf::from(dir_string)
        }
    } else {
        let default_path = home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join("Downloads")
            .display()
            .to_string();

        let dir_string = Text::new("Portal: Where should Portal save this file?")
            .with_default(&default_path)
            .with_help_message("Enter a valid folder path.")
            .prompt()
            .context("No directory provided")?;

        PathBuf::from(dir_string)
    };
    // Ensure the directory exists
    create_dir_all(&target_dir)
        .await
        .context("Failed to create target directory")?;
        Ok(target_dir)
}