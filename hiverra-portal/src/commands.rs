use clap::Subcommand;
use std::path::PathBuf;

// Import anyhow to handle errors without crashing
use anyhow::{Context, Result};

use crate::receiver::receive_file;
use crate::select::select_file_to_send;
use crate::sender::send_file;
use crate::update::update_portal;

// 2. Defining the Choices (The Enum)
// 'pub' makes this visible to main.rs
#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        // The file to send (optional, will prompt if omitted)
        file: Option<PathBuf>,
        // PathBuf, so as to hold "File System Object".
        /// The IP address of the receiver
        #[arg(short, long)]
        address: Option<String>,
        /// The port the receiver is listening on
        #[arg(short, long, default_value_t = 7878)]
        port: u16,
    },
    /// Receive a file
    Receive {
        /// Specify which port to use
        #[arg(short, long, default_value_t = 7878)]
        port: u16,
    },
    /// Update portal to latest version
    Update,
}

impl Commands {
    // This is the method attached to the Enum
    // pub is needed here also to be able to call the function
    // We now return Result<()> to catch errors from sender/receiver
    pub async fn execute(&self) -> Result<()> {
        match self {
            Commands::Send {
                file,
                address,
                port,
            } => {
                // Determine which path to use
                let path_to_send = match file {
                    Some(path) => path.clone(),
                    None => {
                        if let Ok(Some(selected)) = select_file_to_send().await {
                            PathBuf::from(selected)
                        } else {
                            return Ok(());
                        }
                    }
                };

                send_file(&path_to_send, &address, &port)
                    .await
                    .context("Failed to execute Send command")?;
            }
            Commands::Receive { port } => {
                // Pass the error up if receiving fails
                receive_file(&port)
                    .await
                    .context("Failed to execute Receive command")?;
            }
            Commands::Update => {
                update_portal()
                    .await
                    .context("Failed to execute Update commamd")?;
            }
        }
        Ok(()) // Return success if no errors occurred
    }
}
