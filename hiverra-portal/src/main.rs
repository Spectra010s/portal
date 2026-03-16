use {
    clap::Parser,
    commands::Commands,
    std::process::exit,
    tracing::{error, info, trace},
};

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
mod discovery {
    pub mod beacon;
    pub mod listener;
    pub mod protocol;
}
mod logger;
mod progress;

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
    //  Parse the user's input
    let cli = Cli::parse();
    trace!("CLI arguments parsed successfully");

    // start logger
    let _log_guard = logger::init().await;
    trace!("Logger guard initialized");

    info!("Initializing Portal v{}", env!("CARGO_PKG_VERSION"));

    trace!(
        "Executing command: {:?}",
        std::env::args().collect::<Vec<_>>()
    );
    if let Err(e) = cli.command.execute().await {
        error!("Portal Error: {:#}", e);
        eprintln!("Portal Error: {:#}", e);
        // Exit with a non-zero code to tell the OS that the program failed
        exit(1);
    }
}
