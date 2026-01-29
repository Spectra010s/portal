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
}

fn main() {
    // 3. Parse the user's input
    let cli = Cli::parse();

    // 4. Act on the input
    match &cli.command {
        Commands::Send { file } => {
            println!("Portal: Preparing to send '{}'...", file.display());
        }
        Commands::Receive => {
            println!("Portal: Waiting for incoming files...");
        }
    }
}
