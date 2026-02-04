use clap::Subcommand;
use std::path::PathBuf;

use crate::receiver::receive_file;
use crate::sender::send_file;

// 2. Defining the Choices (The Enum)
// 'pub' makes this visible to main.rs
#[derive(Subcommand)]
pub enum Commands {
    /// Send a file
    Send {
        file: PathBuf, // changed it to PathBuf, so as to hold "File System Object".
    },
    /// Receive a file
    Receive,
}

impl Commands {
    // This is the method attached to the Enum
    // pub is needed here also to be able to call the function
    pub fn execute(&self) {
        match self {
            Commands::Send { file } => send_file(&file),
            Commands::Receive => receive_file(),
        }
    }
}
