mod send_item;

use {
    crate::{
        config::models::PortalConfig,
        discovery::listener::find_receiver,
        history::{
            append_record, HistoryItem, HistoryItemKind, HistoryMode, HistoryStatus,
            TransferHistoryRecord,
        },
        metadata::{DirectoryMetadata, FileMetadata, GlobalTransferManifest, TransferItem},
        progress::ProgressManager,
        select::select_files_to_send,
    },
    anyhow::{Context, Result, anyhow},
    async_compression::tokio::write::GzipEncoder,
    async_walkdir::WalkDir,
    bincode::serialize,
    inquire::{Confirm, Text},
    send_item::send_item,
    std::{path::PathBuf,
    time::{Duration,
    Instant}},
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
    total_bytes: u64,
    desc: Option<String>,
    sender_username: Option<String>,
) -> Result<GlobalTransferManifest> {
    debug!(
        "Global Manifest: {} files, {} dirs, {} bytes, sender_username={:?}",
        files, dirs, total_bytes, sender_username
    );
    Ok(GlobalTransferManifest {
        total_files: files,
        total_directories: dirs,
        total_bytes,
        description: desc,
        sender_username,
    })
}

pub async fn start_send(
    file: &Option<Vec<PathBuf>>,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
    recursive: &bool,
) -> Result<()> {
    let mut peer_addr: Option<String> = None;
    let mut peer_username: Option<String> = None;
    let mut start_ts_unix = 0u64;
    let mut start_instant = Instant::now();
    let mut intended_items: Vec<HistoryItem> = Vec::new();
    let mut intended_bytes: u64 = 0;
    let mut sent_items: Vec<HistoryItem> = Vec::new();
    let mut actual_bytes: u64 = 0;

    let result: Result<()> = async {
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
        peer_username = Some(target_username.clone());

        let discovery_result = timeout(
            Duration::from_secs(30),
            find_receiver(&target_username)
        ).await.context("Portal: Search timed out. Make sure the receiver is active and on the same network.")??;

        let (ip, id, p) = discovery_result;
        info!("Receiver found at {}:{} (Node ID: {})", ip, p, id);
        (ip, Some(id), p)
    };

    let r_addr = format!("{}:{}", target_ip, target_port);
    peer_addr = Some(target_ip.clone());
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

    let (file_items, dir_items, calculated_bytes) = items_to_send
        .iter()
        .fold((0u32, 0u32, 0u64), |(f, d, b), (_, item)| match item {
            TransferItem::File(fm) => (f + 1, d, b.saturating_add(fm.file_size)),
            TransferItem::Directory(dm) => (f, d + 1, b.saturating_add(dm.total_size)),
        });
    // Load sender username for manifest
    let sender_username = PortalConfig::load_all()
        .await
        .context("Failed to load sender user config")?
        .user
        .username;
    if sender_username.is_none() {
        warn!("Sender username not set; manifest will omit sender_username");
    } else {
        info!("Sender username loaded for manifest");
    }

    //  Create global manifest
    let global_manifest = create_global_transfer_manifest(
        file_items,
        dir_items,
        calculated_bytes,
        user_desc,
        sender_username.clone(),
    )
    .await?;
    // Start transfer timing when we begin sending the manifest
    start_ts_unix = TransferHistoryRecord::now_unix();
    start_instant = Instant::now();
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

    // progress manager
        let prog = ProgressManager::new();
        debug!("Progress UI created for sender");
        prog.set_total_items(total_items);
        trace!("Progress UI initialized with total_items={}", total_items);

        intended_items = Vec::with_capacity(items_to_send.len());
        intended_bytes = 0;
        sent_items = Vec::with_capacity(items_to_send.len());
        actual_bytes = 0;
        for (_, item) in &items_to_send {
            match item {
                TransferItem::File(fm) => {
                    intended_items.push(HistoryItem {
                        name: fm.filename.clone(),
                        bytes: fm.file_size,
                        kind: HistoryItemKind::File,
                    });
                    intended_bytes = intended_bytes.saturating_add(fm.file_size);
                }
                TransferItem::Directory(dm) => {
                    intended_items.push(HistoryItem {
                        name: dm.dirname.clone(),
                        bytes: dm.total_size,
                        kind: HistoryItemKind::Directory,
                    });
                    intended_bytes = intended_bytes.saturating_add(dm.total_size);
                }
            }
        }
        debug!(
            "History tracker initialized: {} intended items, {} intended bytes",
            intended_items.len(),
            intended_bytes
        );

    debug!("Initializing Gzip encoder and Tar builder...");
    let compressor = GzipEncoder::new(stream);
    let mut builder = Builder::new(compressor);

    info!("Starting TAR stream to network...");
    for (index, (path, item)) in items_to_send.into_iter().enumerate() {
        debug!("Processing item {}: {:?}", index + 1, path);

        // prepare per-file progress bar and pass a clone into send_item
        match item {
            TransferItem::File(fm) => {
                trace!(
                    "Progress UI: starting file item '{}' ({} bytes)",
                    fm.filename,
                    fm.file_size
                );
                prog.set_current_item(index + 1, total_items);
                let filename = fm.filename.clone();
                let file_size = fm.file_size;
                let pb = prog.create_file_bar(&filename, file_size);
                send_item(
                    &mut builder,
                    path,
                    TransferItem::File(fm),
                    Some(pb.clone()),
                )
                    .await
                    .context("Failed to append item to tarball")?;
                pb.finish_and_clear();
                prog.println(format!(
                    "Portal: File '{}' sent successfully!",
                    filename
                ));
                sent_items.push(HistoryItem {
                    name: filename.clone(),
                    bytes: file_size,
                    kind: HistoryItemKind::File,
                });
                actual_bytes = actual_bytes.saturating_add(file_size);
                trace!("Progress UI: completed file item '{}'", filename);
                trace!("History tracker: recorded sent file '{}'", filename);
            }
            TransferItem::Directory(dm) => {
                trace!(
                    "Progress UI: starting directory item '{}' ({} bytes)",
                    dm.dirname,
                    dm.total_size
                );
                prog.set_current_item(index + 1, total_items);
                let dirname = dm.dirname.clone();
                let total_size = dm.total_size;
                if dm.total_size == 0 {
                    prog.println(format!(
                        "Portal: Note: Directory '{}' is empty. Sending structure only.",
                        dirname
                    ));
                }
                let pb = prog.create_file_bar(&dirname, total_size);
                send_item(
                    &mut builder,
                    path,
                    TransferItem::Directory(dm),
                    Some(pb.clone()),
                )
                    .await
                    .context("Failed to append item to tarball")?;
                pb.finish_and_clear();
                prog.println(format!(
                    "Portal: Directory '{}' sent successfully!",
                    dirname
                ));
                sent_items.push(HistoryItem {
                    name: dirname.clone(),
                    bytes: total_size,
                    kind: HistoryItemKind::Directory,
                });
                actual_bytes = actual_bytes.saturating_add(total_size);
                trace!("Progress UI: completed directory item '{}'", dirname);
                trace!("History tracker: recorded sent directory '{}'", dirname);
            }
        }
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

        prog.println("Portal: All file(s) have been sent successfully!");

    let duration_ms = start_instant.elapsed().as_millis() as u64;
    debug!("Preparing successful transfer history record (duration: {}ms)", duration_ms);
    let record = TransferHistoryRecord {
        timestamp: start_ts_unix,
        duration_ms,
        mode: HistoryMode::Send,
        peer_addr: peer_addr.clone(),
        peer_username: peer_username.clone(),
        receiver_path: None,
        description: global_manifest.description.clone(),
        status: HistoryStatus::Success,
        error: None,
        intended_count: total_items as u32,
        intended_bytes,
        intended_items: Some(intended_items.clone()),
        actual_count: sent_items.len() as u32,
        actual_bytes,
        actual_items: Some(sent_items.clone()),
    };
        if let Err(e) = append_record(&record).await {
            warn!("Failed to append history record: {:#}", e);
        } else {
            info!("Successfully appended transfer history record.");
            trace!("Appended success record: {:?}", record);
        }

        Ok(())
    }
    .await;

    if let Err(ref e) = result {
        let duration_ms = start_instant.elapsed().as_millis() as u64;
        debug!("Preparing failed transfer history record (duration: {}ms)", duration_ms);
        let record = TransferHistoryRecord {
            timestamp: start_ts_unix,
            duration_ms,
            mode: HistoryMode::Send,
            peer_addr,
            peer_username,
            receiver_path: None,
            description: None,
            status: HistoryStatus::Failed,
            error: Some(format!("{:#}", e)),
            intended_count: intended_items.len() as u32,
            intended_bytes,
            intended_items: if intended_items.is_empty() {
                None
            } else {
                Some(intended_items)
            },
            actual_count: sent_items.len() as u32,
            actual_bytes,
            actual_items: if sent_items.is_empty() {
                None
            } else {
                Some(sent_items)
            },
        };
        if let Err(err) = append_record(&record).await {
            warn!("Failed to append failed history record: {:#}", err);
        } else {
            info!("Successfully appended failed transfer history record.");
            trace!("Appended failed record details: {:?}", record);
        }
    }

    result
}
