use clap::Subcommand;
use std::{
    fs::{File, metadata},
    io::BufReader,
    path::PathBuf,
};

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
                    // need to be zipped or recursed
                    println!(
                        "Error: '{}' is a directory. Portal only supports single files right now.",
                        file.display()
                    );
                } else {
                    // If it exists AND it's not a directory, it's a file!
                    // Get the metadata (size, permissions, etc.)
                    let file_info = metadata(file).expect("Failed to read metadata");
                    let size = file_info.len(); // Size in bytes
                    // Open the file for reading
                    let Ok(file_handle) = File::open(file) else {
                        println!(
                            "Error: We found the file, but couldn't open it (it might be locked)."
                        );
                        return;
                    };

                    println!("Portal: File found!");

                    println!("Portal: Connection established to the file system.");
                    println!(
                        "Portal: Preparing to send '{}' ({} bytes)...",
                        file.display(),
                        size
                    );
                    let mut reader = BufReader::new(file_handle);

                    println!("Portal: Buffer initialized and ready for streaming.");
                }
            }
            Commands::Receive => {
                println!("Portal: Waiting for incoming files...");
            }
        }
    }
}
