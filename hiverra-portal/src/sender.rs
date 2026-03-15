mod send_item;

use {
    crate::{
        discovery::listener::find_receiver,
        metadata::{DirectoryMetadata, FileMetadata, GlobalTransferManifest, TransferItem},
        select::select_files_to_send,
    },
    anyhow::{Context, Result, anyhow},
    async_compression::tokio::write::GzipEncoder,
    async_walkdir::WalkDir,
    bincode::serialize,
    inquire::{Confirm, Text},
    send_item::send_item,
    std::path::PathBuf,
    std::time::Duration,
    tokio::{
        fs::metadata,
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpStream,
        time::timeout,
    },
    tokio_stream::StreamExt,
    tokio_tar::Builder,
    tracing::{debug, error, info, trace, warn},
};

pub async fn create_file_metadata(path: &PathBuf) -> Result<FileMetadata> {
    let attr = metadata(path).await?;
    debug!("Generating metadata for file: {:?}", path);

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok(FileMetadata {
        filename,
        file_size: attr.len(),
    })
}

async fn create_directory_metadata(dir: &PathBuf) -> Result<DirectoryMetadata> {
    debug!("Calculating total size for directory: {:?}", dir);
    let mut total_size = 0u64;
    let mut entries = WalkDir::new(dir);
    // We walk the directory once to get the total size for the Global Manifest
    while let Some(entry) = entries.next().await {
        let entry = entry.context("Portal: Failed to read directory entry for metadata")?;
        let entry_path = entry.path();
        trace!("Scanning path for size calculation: {:?}", entry_path);

        let file_type = entry.file_type().await?;

        if file_type.is_file() {
            if let Ok(meta) = entry.metadata().await {
                trace!("Found file: {:?} ({} bytes)", entry_path, meta.len());
                total_size += meta.len();
            }
        }
    }

    debug!(
        "Directory size calculation complete: {} bytes total for {:?}",
        total_size, dir
    );
    Ok(DirectoryMetadata {
        dirname: dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_dir")
            .to_string(),
        total_size,
    })
}

pub async fn create_global_transfer_manifest(
    files: u32,
    dirs: u32,
    desc: Option<String>,
) -> Result<GlobalTransferManifest> {
    debug!("Global Manifest: {} files, {} dirs", files, dirs);
    Ok(GlobalTransferManifest {
        total_files: files,
        total_directories: dirs,
        description: desc,
    })
}

pub async fn start_send(
    file: &Option<Vec<PathBuf>>,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
    recursive: &bool,
) -> Result<()> {
    let files = match file {
        Some(path) => path.clone(),
        None => {
            if let Ok(Some(selected)) = select_files_to_send().await {
                selected.clone()
            } else {
                info!("Transfer aborted: No files selected.");
                return Ok(());
            }
        }
    };

    trace!(
        "Validating existence and type of {} input items",
        files.len()
    );
    for file in &files {
        if !file.exists() {
            error!("Path does not exist: {:?}", file);
            return Err(anyhow!(
                "File or directory '{}' does not exist",
                file.display()
            ));
        }
        trace!("Verified path exists: {:?}", file);
        if file.is_dir() {
            if !recursive {
                warn!("Directory encountered without recursive flag: {:?}", file);
                return Err(anyhow!(
                    "-r not specified; omitting directory '{}'",
                    file.display(),
                ));
            }
            trace!("Path is a directory, recursive flag is set.");
        }
    }
    //  Username discovery connection Logic
    let (target_ip, target_node_id, target_port) = if let Some(direct_addr) = addr {
        // Manual override
        info!("Using manual IP address override: {}", direct_addr);
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
        info!("Discovery started for user: {}", target_username);

        let discovery_result = timeout(
            Duration::from_secs(30),
            find_receiver(&target_username)
        ).await.context("Portal: Search timed out. Make sure the receiver is active and on the same network.")??;

        let (ip, id, p) = discovery_result;
        info!("Receiver found at {}:{} (Node ID: {})", ip, p, id);
        (ip, Some(id), p)
    };

    let r_addr = format!("{}:{}", target_ip, target_port);
    println!("Portal: Connecting to {}...", r_addr);

    let mut stream = TcpStream::connect(&r_addr)
        .await
        .context("Could not connect to Receiver!")?;
    info!("TCP connection established with {}", r_addr);
    println!("Portal: Connection established!");

    // Read the ID the receiver is claiming
    debug!("Reading receiver identity proof...");
    let mut id_len_buf = [0u8; 4];
    stream.read_exact(&mut id_len_buf).await?;
    let id_len = u32::from_be_bytes(id_len_buf) as usize;
    trace!("Target claimed ID length: {} bytes", id_len);

    let mut id_buf = vec![0u8; id_len];
    stream.read_exact(&mut id_buf).await?;
    let claimed_id = String::from_utf8(id_buf)?;
    trace!("Target claimed ID string: {}", claimed_id);

    // Verify it matches what we heard in the beacon
    if let Some(expected_id) = target_node_id {
        trace!(
            "Verifying claimed ID against expected beacon ID: {}",
            expected_id
        );
        println!("Portal: Verifying identity...");
        if claimed_id != expected_id {
            error!(
                "SECURITY ALERT: Claimed ID {} does not match beacon ID {}",
                claimed_id, expected_id
            );
            return Err(anyhow!("Portal Security: ID mismatch! Connection aborted."));
        }
        info!("Identity verified via node ID match.");
        println!("Portal: Identity verified. Starting transfer...");
    } else {
        warn!("Direct IP mode used: skipping identity verification.");
        println!(
            "Portal: Connected to {} (Manual mode: Identity check skipped).",
            target_ip
        );
    }
    //  Ask  user if to add a description
    let user_desc = if Confirm::new("Portal: Add description for this transfer?")
        .with_default(false)
        .prompt()?
    {
        let desc = Text::new("Portal: Enter transfer description:").prompt()?;
        info!("User added description: \"{}\"", desc);
        Some(desc)
    } else {
        info!("No description added to transfer.");
        None
    };
    //  Collect all files and directories
    info!("Building item list for transfer...");
    let mut items_to_send: Vec<(PathBuf, TransferItem)> = Vec::new();

    for path in &files {
        trace!("Preparing item: {:?}", path);
        if path.is_dir() {
            let dir_meta = create_directory_metadata(path).await?;
            items_to_send.push((path.clone(), TransferItem::Directory(dir_meta)));
        } else {
            let file_meta = create_file_metadata(path).await?;
            items_to_send.push((path.clone(), TransferItem::File(file_meta)));
        }
    }
    debug!(
        "Successfully collected {} top-level items for manifest",
        items_to_send.len()
    );

    let (file_items, dir_items) = items_to_send
        .iter()
        .fold((0u32, 0u32), |(f, d), (_, item)| match item {
            TransferItem::File(_) => (f + 1, d),
            TransferItem::Directory(_) => (f, d + 1),
        });
    //  Create global manifest
    let global_manifest = create_global_transfer_manifest(file_items, dir_items, user_desc).await?;
    // Serialize and send global manifest
    debug!("Sending serialized global manifest...");
    let encoded_global = serialize(&global_manifest)?;
    let manifest_len = encoded_global.len() as u32;
    trace!("Serialized manifest size: {} bytes", manifest_len);

    stream.write_all(&manifest_len.to_be_bytes()).await?;
    stream.write_all(&encoded_global).await?;

    info!("Global manifest delivered to receiver.");
    println!(
        "Portal: Transfer initialized ({} files, {} folders)",
        file_items, dir_items
    );

    if let Some(d) = &global_manifest.description {
        println!("Portal: Note: {}", d);
        info!("Final manifest description: \"{}\"", d);
    }

    let total_items = items_to_send.len();
    println!("Portal: Preparing to send {} items(s)...", total_items);

    debug!("Initializing Gzip encoder and Tar builder...");
    let compressor = GzipEncoder::new(stream);
    let mut builder = Builder::new(compressor);

    info!("Starting TAR stream to network...");
    for (index, (path, item)) in items_to_send.into_iter().enumerate() {
        println!("Portal: Sending item {} of {}", index + 1, total_items);
        debug!("Processing item {}: {:?}", index + 1, path);

        send_item(&mut builder, path, item)
            .await
            .context("Failed to append item to tarball")?;
    }

    // finalize the Tar archive
    debug!("Finalizing Tar archive structure...");
    builder.finish().await?;

    // get the compressor back
    let mut compressor = builder.into_inner().await?;

    debug!("Shutting down Gzip compressor...");
    compressor
        .shutdown()
        .await
        .context("Failed to shutdown compressor")?;
    trace!("Compressor shutdown complete.");

    // flush the underlying stream to ensure bytes are actually sent
    let mut stream = compressor.into_inner();
    trace!("Flushing underlying TCP stream...");
    stream.flush().await?;
    debug!("TCP stream flush complete.");

    info!(
        "SUCCESS: All {} items sent and stream flushed to {}",
        total_items, r_addr
    );

    println!("Portal: All file(s) have been sent successfully!");

    Ok(())
}
