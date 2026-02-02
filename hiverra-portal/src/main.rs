use clap::{Parser, Subcommand};
use std::path::PathBuf;

// 1. Defining the Map (The Struct)
#[derive(Parser)]
#[command(name = "portal")]
#[command(about = "Hiverra Portal: High-speed file transfer")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// 2. Defining the Choices (The Enum)
#[derive(Subcommand)]
enum Commands {
    /// Send a file
    Send {
        file: PathBuf, // changed it to PathBuf, so as to hold "File System Object".
    },
    /// Receive a file
    Receive,
}impl Commands {
    // This is the method attached to the Enum
    fn execute(&self) {
        match self {
            Commands::Send { file } => {
                // Check 1. check if the path exists before attempting to send
                if !file.exists() {
                    println!("Error: The file  '{}' does not exist.", file.display());
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

fn main() {
    // 3. Parse the user's input
    let cli = Cli::parse();

    // 4. Act on the input
    cli.command.execute();
}
