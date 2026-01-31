# CLI Structure (struct Cli)

This file covers the Structure that handles the initial entry point of the program.

The Cli struct is the entry point for `Hiverra Portal's command-line interface`. It acts as a map that translates raw user input from the terminal into a structured format the program can understand.

## Key Components:
 * #[derive(Parser)]: This macro from the clap crate automatically generates the logic to read std::env::args().
 * Metadata:
   * name: "portal"
   * about: High-speed file transfer description.
   * version: Tracks the current build (currently 0.1.0).
 * The Command Field: It holds a Commands enum, meaning the tool expects a Subcommand (like send or receive) to function.
 
## Responsibility:
To serve as the "Receptionist." It doesn't do the work; it just identifies what the user wants to do and hands it off to the Enum.