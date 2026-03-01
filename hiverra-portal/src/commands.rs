use {
    crate::{
        config::{
            list::list_config, set::set_config, setup::handle_setup, show::show_config_value,
        },
        receiver::start_receiver,
        sender::start_send,
        update::update_portal,
    },
    anyhow::{Context, Result},
    clap::Subcommand,
    std::path::PathBuf,
};

// 2. Defining the Choices (The Enum)
#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        /// The files or folders to send. If empty, opens the interactive picker.
        file: Option<Vec<PathBuf>>,
        /// The IP address of the receiver
        #[arg(short, long)]
        address: Option<String>,
        /// The port the receiver is listening on
        #[arg(short, long, default_value_t = 7878)]
        port: u16,
        /// The username of the receiver
        /// If omitted, Portal will prompt you for a name.
        #[arg(short, long, value_name = "USERNAME")]
        to: Option<String>,
        /// Send folder recursively
        #[arg(short, long, value_name = "FOLDER")]
        recursive: bool,
    },
    /// Receive a file
    Receive {
        /// Specify which port to use
        #[arg(short, long)]
        port: Option<u16>,
        /// Directory where received files will be saved
        #[arg(short, long, value_name = "PATH")]
        dir: Option<PathBuf>,
    },
    /// Update portal to latest version
    Update,
    /// Configuration Settings management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set or Update a setting
    Set {
        /// The configuration key to change
        key: String,
        /// The new value for the setting
        value: String,
    },
    /// View current settings variable value
    Show { key: String },
    /// List all cureent configuration settings
    List,
    /// Initialize or reconfigure Portal's default settings interactively
    Setup,
}

impl Commands {
    // This is the method attached to the Enum
    // return Result<()> to catch errors from sender/receiver
    pub async fn execute(&self) -> Result<()> {
        match self {
            Commands::Send {
                file,
                address,
                port,
                to,
                recursive,
            } => {
                // send file or files
                start_send(&file, &address, &port, &to, &recursive)
                    .await
                    .context("Failed to execute Send command")?;
            }
            Commands::Receive { port, dir } => {
                // Pass the error up if receiving fails
                start_receiver(*port, &dir)
                    .await
                    .context("Failed to execute Receive command")?;
            }
            Commands::Update => {
                update_portal()
                    .await
                    .context("Failed to execute Update commamd")?;
            }
            Commands::Config { action } => match action {
                ConfigAction::Set { key, value } => {
                    set_config(&key, &value)
                        .await
                        .context("Failed to set configuration")?;
                }
                ConfigAction::Show { key } => {
                    // Logic to read and print the a varable value
                    show_config_value(&key)
                        .await
                        .context("Failed to get variable value")?;
                }
                ConfigAction::List => {
                    // Logic to list all the variables
                    list_config().await?;
                }
                ConfigAction::Setup => {
                    handle_setup().await.context("Failed to run setup")?;
                }
            },
        }
        Ok(()) // Return success if no errors occurred
    }
}
