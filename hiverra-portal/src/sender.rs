use std::{
    fs::{File, metadata},
    io::{BufReader, Read, Write, stdin, stdout},
    net::TcpStream,
    path::PathBuf,
};
// Import anyhow for add descriptive error handling
use crate::metadata::FileMetadata;
use anyhow::{Context, Result};
use bincode::serialize;

enum DescChoice {
    Yes,
    No,
    Default,
}

fn create_metadata(file: &PathBuf, desc: Option<String>) -> anyhow::Result<FileMetadata> {
    let attr = metadata(file)?;
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok(FileMetadata {
        filename: name,
        file_size: attr.len(),
        description: desc,
    })
}

pub fn send_file(file: &PathBuf) -> Result<()> {
    // Check 1. check if the path exists before attempting to send
    if !file.exists() {
        println!("Error: The file '{}' does not exist.", file.display());
        return Ok(());
    } else if file.is_dir() {
        // Check 2: Is it a folder
        // We don't want to "send" a folder yet because folders
        // need to be zipped or recursed
        println!(
            "Error: '{}' is a directory. Portal only supports single files right now.",
            file.display()
        );
        return Ok(());
    } else {
        // If it exists AND it's not a directory, it's a file!
        // Get the metadata (size, permissions, etc.)
        // .context() replaces .expect() to provide better error messages without crashing
        println!("Portal: File found!");
        // 1. Ask if user wants to add a description first
        print!("Portal: Add description? (y/N):");
        stdout().flush().context("Failed to flush stdout")?;

        let mut y_n_input = String::new();
        stdin().read_line(&mut y_n_input)?;

        let choice = match y_n_input.trim().to_lowercase().as_str() {
            "y" | "yes" => DescChoice::Yes,
            "n" | "no" => DescChoice::No,
            _ => DescChoice::Default,
        };

        let user_desc = match choice {
            DescChoice::Yes => {
                print!("Portal: Enter a description for '{}': ", file.display());
                stdout().flush().context("Failed to flush stdout")?;

                let mut desc_input = String::new();
                stdin().read_line(&mut desc_input)?;

                Some(desc_input.trim().to_string())
            }
            _ => None,
        };

        // creating metadata with description
        let file_info = create_metadata(file, user_desc).context("Failed to read metadata")?;

        let encoded_metadata = serialize(&file_info).context("Failed to serialize metadata")?;

        let metadata_len = encoded_metadata.len() as u32;

        // Size in bytes
        // Open the file for reading
        let file_handle = File::open(file)
            .context("We found the file, but couldn't open it (it might be locked).")?;

        println!("Portal: Connection established to the file system.");
        println!(
            "Portal: Preparing to send '{}' ({} bytes)...",
            file_info.filename, file_info.file_size
        );
        if let Some(d) = &file_info.description {
            println!("Portal Note: {}", d);
        };
        let mut reader = BufReader::new(file_handle);

        println!("Portal: Buffer initialized and ready for streaming.");

        let mut stream =
            TcpStream::connect("127.0.0.1:7878").context("Could not connect to Reciever!")?;
        println!("Sender: Connected to receiver!");

        // 3. Stream the Metadata to the Pipe
        stream
            .write_all(&metadata_len.to_be_bytes())
            .context("Failed to send metadata length")?;
        stream
            .write_all(&encoded_metadata)
            .context("Failed to send metadata")?;
        let mut buffer = [0u8; 8192];

        // 4. NOW start the File Loop we discussed
        println!("Portal: Sending {}...", file_info.filename);
        loop {
            let bytes_read = reader.read(&mut buffer).context("Failed to read file")?;
            if bytes_read == 0 {
                break;
            }
            stream
                .write_all(&buffer[..bytes_read])
                .context("Failed to send file")?;
        }

        println!("Portal: {} sent successfuly!", file_info.filename);
    }

    Ok(())
}
