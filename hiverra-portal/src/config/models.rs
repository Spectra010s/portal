pub mod network;
pub mod storage;
pub mod user;

use {
    anyhow::{Context, Result, anyhow},
    home::home_dir,
    inquire::{CustomType, Text, validator::Validation},
    network::NetworkConfig,
    rand::random,
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
    storage::StorageConfig,
    tokio::fs,
    user::UserConfig,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PortalConfig {
    pub user: UserConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
}

impl PortalConfig {
    /// Returns the path ~/.portal/
    pub async fn get_dir() -> Result<PathBuf> {
        let mut p = home_dir().ok_or_else(|| anyhow!("Could not find the home directory"))?;
        p.push(".portal");
        Ok(p)
    }

    // forbsettinf config when theres no, values will be updated immediately
    pub fn new_empty_for_set(key: &str, value: &str) -> Result<Self> {
        let mut cfg = PortalConfig {
            user: UserConfig { username: None },
            network: NetworkConfig { default_port: None },
            storage: StorageConfig { download_dir: None },
        };

        // Only fill the key being set
        cfg.update_section(key, value)?;

        Ok(cfg)
    }

    /// Setup Config for first time
    pub async fn interactive_init() -> Result<Self> {
        println!(" Welcome to Portal! Let's get you set up.");

        let suggested_name = format!("puser_{}", random::<u16>());
        // ask for username
        let user_name = Text::new("What is your username?")
            .with_default(&suggested_name)
            .with_help_message(
                "This identifies you during transfers.
                Tip: Press Enter to keep the random suggestion.",
            )
            .with_formatter(&|val| format!("Username: {}@portal", val))
            .prompt()?;

        // ask for port
        let port = CustomType::<u16>::new("Which port should Portal use?")
            .with_default(7878)
            .with_help_message("The local port used for listening. 7878 is recommended.")
            .with_validator(|val: &u16| {
                if *val > 1024 {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        "Port should be > 1024 to avoid system conflicts.".into(),
                    ))
                }
            })
            .prompt()?;

        // ask for downloads directory

        let default_path = home_dir()
            .ok_or_else(|| anyhow!("Home directory not found"))?
            .join("Downloads")
            .display()
            .to_string();

        let dir_string = Text::new("Where should Portal save downloaded files?")
            .with_default(&default_path)
            .with_help_message("Enter a valid folder path.")
            .prompt()?;
        let config = Self {
            user: UserConfig {
                username: if user_name.ends_with("@portal") {
                    Some(user_name.to_string())
                } else {
                    format!("{}@portal", user_name).into()
                },
            },
            network: NetworkConfig {
                default_port: Some(port),
            },
            storage: StorageConfig {
                download_dir: Some(PathBuf::from(dir_string)),
            },
        };

        config.save().await?;

        println!("\n Configuration saved! You're ready to use Portal.");
        Ok(config)
    }

    /// Load from ~/.portal/config.toml or create default

    pub async fn load_or_return() -> Result<Option<Self>> {
        let dir = Self::get_dir().await?;
        let file_path = dir.join("config.toml");

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path)
            .await
            .context("Failed to read config.toml")?;

        let config: Self =
            toml::from_str(&content).context("Syntax error in ~/.portal/config.toml")?;

        Ok(Some(config))
    }

    pub async fn load_all() -> Result<Self> {
        match Self::load_or_return().await? {
            Some(cfg) => Ok(cfg),
            None => Err(anyhow!(
                "No config found. Run 'portal config setup' or 'portal config set <KEY>' to begin."
            )),
        }
    }

    /// Update specific field to configuration file
    pub fn update_section(&mut self, key: &str, value: &str) -> Result<String> {
        //  Split the key by the dot
        let parts: Vec<&str> = key.split('.').collect();
        //  We need at exactly two parts: "section" and "field"
        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid format. Use 'section.field' (e.g., user.username)"
            ));
        }

        let section = parts[0];
        let field = parts[1];

        match section.to_lowercase().as_str() {
            "user" => self.user.update(field, value),
            "network" => self.network.update(field, value),
            "storage" => self.storage.update(field, value),
            _ => Err(anyhow!("Unknown section: '{}'", section)),
        }
    }

    /// Get the value of a specified key
    pub fn get_key_value(&self, key: &str) -> Result<String> {
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() != 2 {
            return Err(anyhow!(
                "Invalid format. Use 'section.field' (e.g., user.username)"
            ));
        }

        let section = parts[0];
        let field = parts[1];

        match section.to_lowercase().as_str() {
            "user" => self.user.get_value(field),
            "network" => self.network.get_value(field),
            "storage" => self.storage.get_value(field),
            _ => Err(anyhow!("Key '{}' not recognized", key)),
        }
    }

    /// Save current config to disk
    pub async fn save(&self) -> Result<()> {
        let dir = Self::get_dir().await?;
        // Ensure the directory exists
        fs::create_dir_all(&dir)
            .await
            .context("Failed to create config directory")?;

        let file_path = dir.join("config.toml");

        let toml_string =
            toml::to_string_pretty(self).context("Failed to format configuration data")?;

        fs::write(file_path, toml_string)
            .await
            .context("Failed to write to config.toml")?;

        Ok(())
    }
}
