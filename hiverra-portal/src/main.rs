use clap::Parser;

mod commands; // Links the commands file
use commands::Commands; // Using the 'pub' enum and fn
mod receiver;
mod sender;

// 1. Defining the Map (The Struct)
#[derive(Parser)]
#[command(name = "portal")]
#[command(about = "Hiverra Portal: High-speed file transfer")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    // 3. Parse the user's input
    let cli = Cli::parse();
    // 4. Act on the input
    cli.command.execute();
}
