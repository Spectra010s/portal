use {
    crate::{
        discovery::listener::find_receiver,
        metadata::{FileMetadata, TransferManifest},
        select::select_files_to_send,
    },
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

async fn create_metadata(file: &PathBuf) -> Result<FileMetadata> {
    let attr = metadata(file).await?;
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok(FileMetadata {
        filename: name,
        file_size: attr.len(),
    })
}

async fn create_tf_manifest(files: &[PathBuf], desc: Option<String>) -> Result<TransferManifest> {
    let mut metadata_list = Vec::new();

    for file in files {
        let meta = create_metadata(file).await?;
        metadata_list.push(meta);
    }

    Ok(TransferManifest {
        total_files: metadata_list.len() as u32,
        files: metadata_list,
        description: desc,
    })
}

pub async fn send_file(
    file: &Option<Vec<PathBuf>>,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
) -> Result<()> {
    let files = match file {
        Some(path) => path.clone(),
        None => {
            if let Ok(Some(selected)) = select_files_to_send().await {
                selected.clone()
            } else {
                return Ok(());
            }
        }
    };

    for file in &files {
        if !file.exists() {
            return Err(anyhow!("File '{}' does not exist", file.display()));
        }

        if file.is_dir() {
            return Err(anyhow!(
                "'{}' is a directory. Portal only supports single files right now.",
                file.display()
            ));
        }
    }

    //  Ask  user if to add a description
    let user_desc = if Confirm::new("Portal: Add description for this transfer?")
        .with_default(false)
        .prompt()?
    {
        Some(Text::new("Portal: Enter transfer description:").prompt()?)
    } else {
        None
    };

    let manifest = create_tf_manifest(&files, user_desc)
        .await
        .context("Failed to build transfer manifest")?;

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

    // Tsart the serialization of the manifest
    let encoded_manifest = serialize(&manifest).context("Failed to serialize manifest")?;
    let manifest_len = encoded_manifest.len() as u32;

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

    println!(
        "Portal: Preparing to send {} file(s)...",
        manifest.total_files
    );

    if let Some(d) = &manifest.description {
        println!("Portal: Note: {}", d);
    }

    // Stream the manifest to the Pipe
    stream
        .write_all(&manifest_len.to_be_bytes())
        .await
        .context("Failed to send manifest length")?;
    stream
        .write_all(&encoded_manifest)
        .await
        .context("Failed to send manifest")?;

    println!("Portal: Manifest sent.");

    // Send files sequentially
    for (index, (path, file_info)) in files.iter().zip(&manifest.files).enumerate() {
        println!(
            "Portal: Preparing to send '{}' ({} bytes)...",
            file_info.filename, file_info.file_size
        );
        println!(
            "Portal: Sending file {} of {}",
            index + 1,
            manifest.total_files
        );

        let file_handle = File::open(path)
            .await
            .context("We found the file, but couldn't open it (it might be locked).")?;

        println!("Portal: Connection established to the file system.");

        let mut reader = BufReader::new(file_handle);
        println!("Portal: Buffer initialized and ready for streaming.");

        let mut buffer = [0u8; 8192];

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

        println!("Portal: File '{}' sent successfully!", file_info.filename,);
    }
    println!("Portal: All file(s) have been sent successfully!");

    Ok(())
}
