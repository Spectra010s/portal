use clap::Parser;
use std::process::exit;

mod commands; // Links the commands file
use commands::Commands; // Using the 'pub' enum and fn
mod metadata;
mod receiver;
mod sender;

// 1. Defining the Map (The Struct)
#[derive(Parser)]
#[command(name = "portal")]
#[command(about = "Hiverra Portal: High-speed file transfer")]
#[command(version = "0.3.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    // 3. Parse the user's input
    let cli = Cli::parse();

    // 4. Act on the input
    // Since execute() now returns a Result, we check if it's an Error (Err)
    if let Err(e) = cli.command.execute() {
        // eprint! prints to the 'Standard Error' stream instead of 'Standard Output'
        // {:?} prints the error message plus all the .context() notes we added
        eprintln!("Portal Error: {:#}", e);

        // Exit with a non-zero code to tell the OS that the program failed
        exit(1);
    }
}
