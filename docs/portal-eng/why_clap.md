# Choice of CLI Framework: Clap
For Hiverra Portal, we chose the clap (Command Line Argument Parser) crate to handle user input rather than parsing std::env::args() manually.

## Why Clap?
 * Standardization: Clap follows the POSIX standard for CLI tools. It handles flags (-v), options (--file), and subcommands automatically.
 * Auto-Generated Help: By just writing our code, Clap automatically generates a --help menu. This saves us from writing hundreds of lines of "Help" text manually.
 * Type Safety: It integrates perfectly with Rust’s type system. It doesn't just give us a "String"; it maps input directly into our Cli struct and Commands enum.
 * Validation: Clap handles basic errors (like a user forgetting a required argument) before our main() function even runs.
 
### Alternatives Considered:
 * Manual Parsing: Too error-prone and tedious for a project that plans to scale.
 * StructOpt: Now merged into Clap v3+, making Clap the industry standard for Rust.
 
## Responsibility:
To act as the Foundation for the user interface. It ensures that the "Portal" feels like a professional, polished tool from the very first command.