mod handshake;
mod manifest;
mod send_item;
mod stream;

pub use manifest::create_file_metadata;

use {
    crate::{
        config::models::PortalConfig,
        history::{
            append_record, HistoryItem, HistoryItemKind, HistoryMode, HistoryStatus,
            TransferHistoryRecord,
        },
        metadata::TransferItem,
        progress::ProgressManager,
        select::select_files_to_send,
    },
    stream::send_stream,
    handshake::connect_and_verify,
    manifest::{create_directory_metadata, create_global_transfer_manifest},
    anyhow::{Context, Result, anyhow},
    bincode::serialize,
    inquire::{Confirm, Text},
    std::{path::PathBuf,
    time::Instant},
    tokio::{
        io::AsyncWriteExt,
        net::TcpStream,
    },
    tracing::{debug, error, info, trace, warn},
};

pub async fn start_send(
    file: &Option<Vec<PathBuf>>,
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
    recursive: &bool,
    no_compress: &bool,
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
    let (stream, connected_r_addr, connected_addr, connected_username) =
        connect_and_verify(addr, port, to).await?;
    let mut stream: TcpStream = stream;
    peer_addr = connected_addr;
    peer_username = connected_username;
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
    let compressed = !*no_compress;
    let global_manifest = create_global_transfer_manifest(
        file_items,
        dir_items,
        calculated_bytes,
        user_desc,
        sender_username.clone(),
        compressed,
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

    send_stream(
        stream,
        items_to_send,
        &prog,
        total_items,
        &mut sent_items,
        &mut actual_bytes,
        *no_compress,
    )
    .await?;

    info!(
        "SUCCESS: All {} items sent and stream flushed to {}",
        total_items, connected_r_addr
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
