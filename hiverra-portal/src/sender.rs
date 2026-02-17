use {
    crate::discovery::listener::find_receiver,
    crate::metadata::FileMetadata,
    anyhow::{Context, Result, anyhow},
    bincode::serialize,
    inquire::{Confirm, Text},
    std::path::PathBuf,
    std::time::Duration,
    tokio::{
        fs::{File, metadata},
        io::{AsyncReadExt, AsyncWriteExt, BufReader},
        net::TcpStream,
        time::timeout,
    },
};

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

pub async fn send_file(
    file: &PathBuf,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
) -> Result<()> {
    // Check 1. check if the path exists before attempting to send
    if !file.exists() {
        println!("Error: The file '{}' does not exist.", file.display());
        return Ok(());
    } else if file.is_dir() {
        // Check 2: Is it a folder
        println!(
            "Error: '{}' is a directory. Portal only supports single files right now.",
            file.display()
        );
        return Ok(());
    } else {
        // If it exists AND it's not a directory, it's a file!
        println!("Portal: File found!");

        //  New username discovery connection Logic
        let (target_ip, target_node_id, target_port) = if let Some(direct_addr) = addr {
            // Manual override
            (direct_addr.clone(), None, *port)
        } else {
            // Discovery Mode
            let target_username = match to {
                Some(username) => username.clone(),
                None => Text::new("Portal: Enter Receiver's username:")
                    .prompt()
                    .context("Failed to get username")?,
            };

            println!("Portal: Searching for receiver...: {}", target_username);

            let discovery_result = timeout(
        Duration::from_secs(30),
        find_receiver(&target_username)
    ).await.context("Portal: Search timed out. Make sure the receiver is active and on the same network.")??;

            let (ip, id, p) = discovery_result;
            (ip, Some(id), p)
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
        let file_info = create_metadata(file, user_desc)
            .await
            .context("Failed to read metadata")?;

        let encoded_metadata = serialize(&file_info).context("Failed to serialize metadata")?;
        let metadata_len = encoded_metadata.len() as u32;

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

        let r_addr = format!("{}:{}", target_ip, target_port);
        println!("Portal: Connecting to {}...", r_addr);

        let mut stream = TcpStream::connect(&r_addr)
            .await
            .context("Could not connect to Receiver!")?;
        println!("Portal: Connection established!");

        // Read the ID the receiver is claiming
        let mut id_len_buf = [0u8; 4];
        stream.read_exact(&mut id_len_buf).await?;
        let id_len = u32::from_be_bytes(id_len_buf) as usize;

        let mut id_buf = vec![0u8; id_len];
        stream.read_exact(&mut id_buf).await?;
        let claimed_id = String::from_utf8(id_buf)?;

        // Verify it matches what we heard in the beacon
        if let Some(expected_id) = target_node_id {
            println!("Portal: Verifying identity...");
            if claimed_id != expected_id {
                return Err(anyhow!("Portal Security: ID mismatch! Connection aborted."));
            }
            println!("Portal: Identity verified. Starting transfer...");
        } else {
            println!(
                "Portal: Connected to {} (Manual mode: Identity check skipped).",
                target_ip
            );
        }

        // Stream the Metadata to the Pipe
        stream
            .write_all(&metadata_len.to_be_bytes())
            .await
            .context("Failed to send metadata length")?;
        stream
            .write_all(&encoded_metadata)
            .await
            .context("Failed to send metadata")?;

        let mut buffer = [0u8; 8192];

        // NOW start the File Loop
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

        println!("Portal: {} sent successfully!", file_info.filename);
    }

    Ok(())
}
