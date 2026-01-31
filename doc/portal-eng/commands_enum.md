# Commands Enum (enum Commands)

This part is crucial because it defines the actual "modes" of the Portal.

The Commands enum represents the different choices or subcommands available in Portal. In Rust, an Enum is perfect for this because it ensures the user can only pick one valid mode at a time.

## Available Variants:
 * Send: Used when the user wants to push a file out of the portal.
   * Data: It carries a file field of type PathBuf.
   * Why PathBuf?: It is a cross-platform way to handle file system paths (works on Windows, Mac, and Linux).
 * Receive: Used when the user is waiting to catch a file.
   * Data: Currently holds no data (a "Unit" variant), as it simply puts the app into listening mode.
   
## Key Components:
 * #[derive(Subcommand)]: This tells clap that each variant in this Enum should be treated as a standalone command in the terminal (e.g., portal send vs portal receive).
 * Documentation Comments (///): The comments inside the code are used by clap to automatically generate the "help" menu when a user types portal --help.
 
## Responsibility:
To define the Vocabulary of the application. It acts as the "Menu" of what the portal is capable of doing.