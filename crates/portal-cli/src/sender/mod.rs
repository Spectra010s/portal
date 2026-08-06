mod history;

use {
    crate::{
        config::models::PortalConfig,
        history::{
            HistoryItem, HistoryItemKind, HistoryStatus, TransferHistoryRecord, append_record,
        },
        progress::ProgressManager,
        select::select_files_to_send,
    },
    anyhow::{Context, Result, anyhow},
    history::build_history_record,
    inquire::{Confirm, Text},
    pxp::metadata::TransferItem,
    std::{path::PathBuf, time::Instant},
    tokio::net::TcpStream,
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

        // --- Connection ---
        let (target_ip, target_node_id, target_port) = if let Some(direct_addr) = addr {
            info!("Using manual IP address override: {}", direct_addr);
            (direct_addr.clone(), None, *port)
        } else {
            let target_username = match to {
                Some(username) => username.clone(),
                None => Text::new("Portal: Enter Receiver's username:")
                    .prompt()
                    .context("Failed to get username")?,
            };

            println!("Portal: Searching for receiver...: {}", target_username);
            peer_username = Some(target_username.clone());

            let (ip, id, p) = pxp::sender::discover_receiver(&target_username, *port).await?;
            (ip, Some(id), p)
        };

        let r_addr = format!("{}:{}", target_ip, target_port);
        peer_addr = Some(target_ip.clone());
        println!("Portal: Connecting to {}...", r_addr);

        let mut stream: TcpStream = pxp::sender::connect_to_receiver(
            &target_ip,
            target_port,
            target_node_id.as_deref(),
        )
        .await?;

        println!("Portal: Connection established!");
        if target_node_id.is_some() {
            println!("Portal: Verifying identity...");
            println!("Portal: Identity verified. Starting transfer...");
        } else {
            println!(
                "Portal: Connected to {} (Manual mode: Identity check skipped).",
                target_ip
            );
        }

        // --- Description ---
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

        // --- Build item list ---
        info!("Building item list for transfer...");
        let mut items_to_send: Vec<(PathBuf, TransferItem)> = Vec::new();

        for path in &files {
            trace!("Preparing item: {:?}", path);
            if path.is_dir() {
                let dir_meta = pxp::sender::create_directory_metadata(path).await?;
                items_to_send.push((path.clone(), TransferItem::Directory(dir_meta)));
            } else {
                let file_meta = pxp::sender::create_file_metadata(path).await?;
                items_to_send.push((path.clone(), TransferItem::File(file_meta)));
            }
        }
        debug!(
            "Successfully collected {} top-level items for manifest",
            items_to_send.len()
        );

        let (file_items, dir_items, calculated_bytes) =
            items_to_send
                .iter()
                .fold((0u32, 0u32, 0u64), |(f, d, b), (_, item)| match item {
                    TransferItem::File(fm) => (f + 1, d, b.saturating_add(fm.file_size)),
                    TransferItem::Directory(dm) => (f, d + 1, b.saturating_add(dm.total_size)),
                });

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

        // --- Create and send manifest ---
        let compressed = !*no_compress;
        let global_manifest = pxp::sender::create_global_transfer_manifest(
            file_items,
            dir_items,
            calculated_bytes,
            user_desc,
            sender_username.clone(),
            compressed,
        )
        .await?;

        start_ts_unix = TransferHistoryRecord::now_unix();
        start_instant = Instant::now();

        pxp::sender::send_manifest(&mut stream, &global_manifest).await?;

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

        // --- Progress + history tracking ---
        let prog = ProgressManager::new();
        debug!("Progress UI created for sender");
        prog.set_total_items(total_items);

        intended_items = Vec::with_capacity(items_to_send.len());
        intended_bytes = 0;
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

        // Build sent_items from the items_to_send list (all will be sent if successful)
        sent_items = intended_items.clone();
        actual_bytes = intended_bytes;

        // --- Send stream using core ---
        pxp::sender::send_stream(
            stream,
            items_to_send,
            *no_compress,
            Some(&prog as &dyn pxp::TransferProgress),
        )
        .await?;

        info!(
            "SUCCESS: All {} items sent and stream flushed to {}",
            total_items, r_addr
        );

        prog.println("Portal: All file(s) have been sent successfully!");

        let duration_ms = start_instant.elapsed().as_millis() as u64;
        debug!(
            "Preparing successful transfer history record (duration: {}ms)",
            duration_ms
        );
        let record = build_history_record(
            start_ts_unix,
            duration_ms,
            HistoryStatus::Success,
            peer_addr.clone(),
            peer_username.clone(),
            global_manifest.description.clone(),
            intended_items.clone(),
            intended_bytes,
            sent_items.clone(),
            actual_bytes,
        );
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
        debug!(
            "Preparing failed transfer history record (duration: {}ms)",
            duration_ms
        );
        let mut record = build_history_record(
            start_ts_unix,
            duration_ms,
            HistoryStatus::Failed,
            peer_addr,
            peer_username,
            None,
            intended_items,
            intended_bytes,
            sent_items,
            actual_bytes,
        );
        record.error = Some(format!("{:#}", e));
        if let Err(err) = append_record(&record).await {
            warn!("Failed to append failed history record: {:#}", err);
        } else {
            info!("Successfully appended failed transfer history record.");
            trace!("Appended failed record details: {:?}", record);
        }
    }

    result
}
