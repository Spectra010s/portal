use clap::Parser;
use std::process::exit;

// link files
mod commands;
mod config {
    pub mod list;
    pub mod models;
    pub mod set;
    pub mod setup;
    pub mod show;
}
mod metadata;
mod receiver;
mod select;
mod sender;
mod update;
use commands::Commands;

// 1. Defining the Map (The Struct)
#[derive(Parser)]
#[command(name = "portal")]
#[command(about = "Hiverra Portal: A lightweight CLI tool to transfer files between devices.")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    // 3. Parse the user's input
    let cli = Cli::parse();

    // 4. Act on the input
    // Since execute() now returns a Result, we check if it's an Error (Err)
    if let Err(e) = cli.command.execute().await {
        // eprint! prints to the 'Standard Error' stream instead of 'Standard Output'
        // {:?} prints the error message plus all the .context() notes we added
        eprintln!("Portal Error: {:#}", e);

        // Exit with a non-zero code to tell the OS that the program failed
        exit(1);
    }
}
