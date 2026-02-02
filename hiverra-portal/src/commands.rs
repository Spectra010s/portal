use std::path::PathBuf;
use clap::Subcommand;

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
            Commands::Send { file } => {
                // Check 1. check if the path exists before attempting to send
                if !file.exists() {
                    println!("Error: The file '{}' does not exist.", file.display());
                } else if file.is_dir() {
               // Check 2: Is it a folder? 
               // We don't want to "send" a folder yet because folders 
               // need to be zipped or recursed.
                    println!("Error: '{}' is a directory. Portal only supports single files right now.", file.display());
                } else {
                    // If it exists AND it's not a directory, it's a file!
                    println!("Portal: File found! Preparing to send '{}'...", file.display());
                }
            }
            Commands::Receive => {
                println!("Portal: Waiting for incoming files...");
            }
        }
    }
}
