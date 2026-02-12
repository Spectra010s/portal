use anyhow::{Context, Result, anyhow};
use home::home_dir;
use inquire::{CustomType, Text, validator::Validation};
use rand::random;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct PortalConfig {
    pub default_port: u16,
    pub username: String,
}

impl PortalConfig {
    /// Returns the path ~/.portal/
    pub async fn get_dir() -> Result<PathBuf> {
        let mut p = home_dir().ok_or_else(|| anyhow!("Could not find the home directory"))?;
        p.push(".portal");
        Ok(p)
    }
    /// Setup Config for first time
    pub async fn interactive_init() -> Result<Self> {
        println!(" Welcome to Portal! Let's get you set up.");

        let suggested_name = format!("puser_{}", random::<u16>());

        let username = Text::new("What is your username?")
            .with_default(&suggested_name)
            .with_help_message(
                "This identifies you during transfers.
    Tip: Press Enter to keep the random suggestion.",
            )
            .with_formatter(&|val| format!("Username: {}@portal", val))
            .prompt()?;

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

        let config = Self {
            username: format!("{}@portal", username),
            default_port: port,
        };

        config.save().await?;
        println!("\n Configuration saved! You're ready to use Portal.");
        Ok(config)
    }

    /// Load from ~/.portal/config.toml or create default
    pub async fn load() -> Result<Self> {
        let dir = Self::get_dir().await?;
        let file_path = dir.join("config.toml");

        // Check if the file exists. If its not, tell user to setup
        if !file_path.exists() {
            if !dir.exists() {
                fs::create_dir_all(&dir)
                    .await
                    .context("Failed to create configuration directory")?;
            }

            // Trigger the wizard and return its result
            return Err(anyhow!(
                "No config found. Run 'portal config setup' or 'portal config set <KEY>' to begin."
            ));
        }

        // If we reached here, then the file exists. Read it.
        let content = fs::read_to_string(&file_path)
            .await
            .context("Failed to read config.toml")?;

        let config: Self =
            toml::from_str(&content).context("Syntax error in ~/.portal/config.toml")?;

        Ok(config)
    }
    /// Update specific field to config
    pub fn update_field(&mut self, key: &str, value: &str) -> Result<String> {
        match key.to_lowercase().as_str() {
            "port" => {
                self.default_port = value
                    .parse()
                    .context("Invlaid Port number: Port must be a number")?;
                Ok(self.default_port.to_string())
            }
            "username" => {
                self.username = if value.ends_with("@portal") {
                    value.to_string()
                } else {
                    format!("{}@portal", value)
                };
                Ok(self.username.clone())
            }
            _ => Err(anyhow!("Unknown config key: {}", key)),
        }
    }

    /// Save current config to disk
    pub async fn save(&self) -> Result<()> {
        let file_path = Self::get_dir().await?.join("config.toml");

        let toml_string =
            toml::to_string_pretty(self).context("Failed to format configuration data")?;

        fs::write(file_path, toml_string)
            .await
            .context("Failed to write to config.toml")?;

        Ok(())
    }
}
