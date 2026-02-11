use tokio::{
    fs::{File, metadata},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, stdin, stdout},
    net::TcpStream,
};

use std::path::PathBuf;
// Import anyhow for add descriptive error handling
use crate::metadata::FileMetadata;
use anyhow::{Context, Result};
use bincode::serialize;

enum DescChoice {
    Yes,
    No,
    Default,
}

async fn create_metadata(file: &PathBuf, desc: Option<String>) -> anyhow::Result<FileMetadata> {
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

pub async fn send_file(file: &PathBuf, addr: &str) -> Result<()> {
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
        stdout().flush().await.context("Failed to flush stdout")?;

        let mut stdin_reader = BufReader::new(stdin());

        let mut y_n_input = String::new();
        stdin_reader.read_line(&mut y_n_input).await?;

        let choice = match y_n_input.trim().to_lowercase().as_str() {
            "y" | "yes" => DescChoice::Yes,
            "n" | "no" => DescChoice::No,
            _ => DescChoice::Default,
        };

        let user_desc = match choice {
            DescChoice::Yes => {
                print!("Portal: Enter a description for '{}': ", file.display());
                stdout().flush().await.context("Failed to flush stdout")?;

                let mut desc_input = String::new();
                stdin_reader.read_line(&mut desc_input).await?;

                Some(desc_input.trim().to_string())
            }
            _ => None,
        };

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
        let r_addr = format!("{}:7878", addr);
        println!("Portal: connecting to {}", r_addr);
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
