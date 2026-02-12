use inquire::{Confirm, Text};
use std::path::PathBuf;
use tokio::{
    fs::{File, metadata},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
// Import anyhow for add descriptive error handling
use crate::metadata::FileMetadata;
use anyhow::{Context, Result};
use bincode::serialize;

async fn create_metadata(file: &PathBuf, desc: Option<String>) -> Result<FileMetadata> {
    let attr = metadata(file).await?;
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

pub async fn send_file(file: &PathBuf, addr: &Option<String>, port: &u16) -> Result<()> {
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
        println!("Portal: File found!");

        // Ask user for reciver addrress
        let re_addr = match addr {
            Some(address) => address.clone(),
            None => Text::new("Portal: Enter Recieiver's address:")
                .prompt()
                .context("Failed to get address")?,
        };

        // 2. Ask if user wants to add a description first

        let user_desc = if Confirm::new("Portal: Add description?")
            .with_default(false)
            .prompt()?
        {
            let desc_msg = format!("Portal: Enter a description for '{}': ", file.display());

            let desc_input = Text::new(&desc_msg).prompt()?;

            Some(desc_input)
        } else {
            None
        };

        // Get the metadata (size, permissions, etc.)
        // creating metadata with description
        let file_info = create_metadata(file, user_desc)
            .await
            .context("Failed to read metadata")?;

        let encoded_metadata = serialize(&file_info).context("Failed to serialize metadata")?;

        let metadata_len = encoded_metadata.len() as u32;

        // Size in bytes
        // Open the file for reading
        let file_handle = File::open(file)
            .await
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
        let r_addr = format!("{}:{}", re_addr, port);
        println!("Portal: Connecting to {}", r_addr);
        let mut stream = TcpStream::connect(r_addr)
            .await
            .context("Could not connect to Reciever!")?;
        println!("Sender: Connected to receiver!");

        // 3. Stream the Metadata to the Pipe
        stream
            .write_all(&metadata_len.to_be_bytes())
            .await
            .context("Failed to send metadata length")?;
        stream
            .write_all(&encoded_metadata)
            .await
            .context("Failed to send metadata")?;
        let mut buffer = [0u8; 8192];

        // 4. NOW start the File Loop we discussed
        println!("Portal: Sending {}...", file_info.filename);
        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .context("Failed to read file")?;
            if bytes_read == 0 {
                break;
            }
            stream
                .write_all(&buffer[..bytes_read])
                .await
                .context("Failed to send file")?;
        }

        println!("Portal: {} sent successfuly!", file_info.filename);
    }

    Ok(())
}
