mod get_dir;
mod local_ip;
mod receive_item;

use {
    crate::{
        config::models::PortalConfig,
        discovery::beacon::start_beacon,
        history::{append_record, HistoryMode, HistoryStatus, TransferHistoryRecord},
        metadata::GlobalTransferManifest,
        progress::{ProgressManager, Side},
    },
    anyhow::{Context, Result, anyhow},
    async_compression::tokio::bufread::GzipDecoder,
    bincode::deserialize,
    get_dir::get_target_dir,
    local_ip::get_local_ip,
    receive_item::receive_item,
    std::{path::PathBuf,
    time::Instant, },
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt, BufReader},
        net::TcpListener,
    },
    tokio_tar::Archive,
    tracing::{debug, error, info, trace},
    uuid::Uuid,
};

pub async fn start_receiver(port: Option<u16>, dir: &Option<PathBuf>) -> Result<()> {
    info!("Portal: Initializing receiver systems...");
    let mut peer_addr: Option<String> = None;
    let mut start_ts_unix = 0u64;
    let mut start_instant = Instant::now();
    let mut expected_items: Option<u32> = None;

    let result: Result<()> = async {

    let my_ip = get_local_ip()
        .await
        .context("Failed to get IP address, pls try again")?;
    debug!("Local IP detected: {:?}", my_ip);

    // Use the CLI flag directly
    let n_port = if let Some(port) = port {
        trace!("Port source: CLI argument");
        debug!("Portal: Overriding config port with CLI port: {}", port);
        port
    } else if let Some(cfg) = PortalConfig::load_or_return().await? {
        //  Use config if it exists and has a value
        if let Some(p) = cfg.network.default_port {
            trace!("Port source: User Configuration");
            debug!("Portal: Port not given, using config port: {}", p);
            p
        } else {
            error!("Port missing in both CLI and config");
            return Err(anyhow!("No port provided and config has no port set"));
        }
    } else {
        //  Neither CLI nor config
        trace!("Port source: No configuration found, falling back to requirement check");
        error!("No port configuration found");
        return Err(anyhow!("No port provided and no config found"));
    };

    // Fetching username with load_all
    let full_cfg = PortalConfig::load_all()
        .await
        .context("Failed to load user config")?;

    let username = full_cfg.user.username.ok_or_else(|| {
        error!("Attempted to receive without a username set");
        anyhow!("No username found. Please run 'portal config set user.username <name>' first.")
    })?;

    // Unique session ID for this transfer
    let node_id = Uuid::new_v4().to_string();
    debug!("Generated session Node ID: {}", node_id);

    let new_addr = format!("0.0.0.0:{}", n_port);
    trace!("Listener target address: {}", new_addr);

    let listener = TcpListener::bind(&new_addr)
        .await
        .context("Failed to bind to port")?;

    println!("Portal: Creating wormhole at {:?}", my_ip);
    println!("Portal: Wormhole open for {:?}", username);
    info!("TCP Listener bound to {}", new_addr);

    // The Tokio Select logic to run Beacon and Listener together
    let (mut socket, addr) = tokio::select! {
        // start the beacon
        _ = start_beacon(username, node_id.clone(), n_port) => {
            error!("Discovery beacon exited unexpectedly");
            return Err(anyhow!("Portal: Discovery beacon stopped unexpectedly"));
        }
        // wait for the actual TCP connection
        result = listener.accept() => {
            let (conn, addr) = result.context("Failed to accept connection")?;
            trace!("Accepted raw TCP connection from: {:?}", addr);
            (conn, addr)
        }
    };

    info!("Connection accepted from sender: {}", addr);
    peer_addr = Some(addr.ip().to_string());
    println!("Portal: Connection established with {}!", addr);
    println!("Portal: Connected to sender");
    println!("Portal: Waiting for incoming files...");

    // Send ID to Sender so they can verify who we are
    debug!("Sending Node ID for verification: {}", node_id);
    let id_bytes = node_id.as_bytes();
    let id_len = id_bytes.len() as u32;
    trace!("Node ID length: {} bytes", id_len);

    socket
        .write_all(&id_len.to_be_bytes())
        .await
        .context("Failed to send verification length")?;
    socket
        .write_all(id_bytes)
        .await
        .context("Failed to send verification ID")?;
    trace!("Verification identity sent to peer.");

    // Start transfer timing when we begin receiving the manifest
    start_ts_unix = TransferHistoryRecord::now_unix();
    start_instant = Instant::now();
    //  Read the metadata length
    let mut global_manifest_len_buf = [0u8; 4];
    socket
        .read_exact(&mut global_manifest_len_buf)
        .await
        .context("Failed to read global manifest length")?;

    let global_manifest_len = u32::from_be_bytes(global_manifest_len_buf) as usize;
    debug!(
        "Incoming global manifest length: {} bytes",
        global_manifest_len
    );

    //  Read the Metadata Blob
    let mut global_manifest_buf = vec![0u8; global_manifest_len];
    socket
        .read_exact(&mut global_manifest_buf)
        .await
        .context("Failed to read global manifest blob")?;
    trace!(
        "Read global manifest raw bytes (size: {}). Deserializing...",
        global_manifest_len
    );

    // Deserialize the manifest
    let global_manifest: GlobalTransferManifest =
        deserialize(&global_manifest_buf).context("Failed to deserialize global manifest")?;

    info!("Global manifest received and deserialized successfully.");
    trace!("Manifest data: {:?}", global_manifest);

    let total_directories = &global_manifest.total_directories;
    let total_files = global_manifest.total_files;
    let description = global_manifest.description.clone();

    let total_items = total_files + total_directories;
    expected_items = Some(total_items);

    // Print basic info for the user
    println!("Portal: Incoming transfer - {} item(s)", total_items);

    if let Some(desc) = &description {
        println!("Portal: Sender left a note: \"{}\"", desc);
        info!("Transfer Note: {}", desc);
    } else {
        info!("Transfer has no description.");
    }

    // Determine the directory to save files
    let target_dir = get_target_dir(&dir).await?;
    info!("Target directory for saving: {:?}", target_dir);

    // receive file or directories
    debug!("Initializing Gzip decoder and Tar archive reader...");
    let reader = BufReader::new(socket);
    let decoder = GzipDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    // progress manager for receiver UI
    let prog = ProgressManager::new_with_side(Side::Receiver);
    debug!("Progress UI created for receiver");
    prog.set_total_items(total_items as usize);
    trace!("Progress UI initialized with total_items={}", total_items);

    let summary =
        receive_item(&mut archive, &target_dir, total_items, Some(prog.clone())).await?;
    trace!("receive_item recursive loop completed.");

    debug!("Extraction complete. Recovering stream...");
    let decoder = archive.into_inner().map_err(|_| {
        error!("Failed to recover decoder from archive");
        anyhow!("Failed to recover decoder from archive")
    })?;
    trace!("GzipDecoder recovered from Archive wrapper.");

    let _socket = decoder.into_inner();
    trace!("TcpStream recovered from GzipDecoder.");

    info!(
        "SUCCESS: Transfer completed. Saved to {}",
        target_dir.display()
    );
    prog.println(format!(
        "Portal: All item(s) have been received successfully! Saved to '{}'",
        target_dir.display()
    ));

    let duration_ms = start_instant.elapsed().as_millis() as u64;
    debug!("Preparing successful receive history record (duration: {}ms)", duration_ms);
    let record = TransferHistoryRecord {
        timestamp: start_ts_unix,
        duration_ms,
        mode: HistoryMode::Receive,
        peer_addr: peer_addr.clone(),
        peer_username: None,
        receiver_path: Some(target_dir.display().to_string()),
        description: description.clone(),
        status: HistoryStatus::Success,
        error: None,
        intended_count: expected_items.unwrap_or(summary.items.len() as u32),
        intended_bytes: 0,
        intended_items: None,
        actual_count: summary.items.len() as u32,
        actual_bytes: summary.total_bytes,
        actual_items: Some(summary.items),
    };
    if let Err(e) = append_record(&record).await {
        error!("Failed to append history record: {:#}", e);
    } else {
        info!("Successfully appended receive history record.");
        trace!("Appended success record: {:?}", record);
    }

    Ok(())
    }
    .await;

    if let Err(ref e) = result {
        let duration_ms = start_instant.elapsed().as_millis() as u64;
        debug!("Preparing failed receive history record (duration: {}ms)", duration_ms);
        let record = TransferHistoryRecord {
            timestamp: start_ts_unix,
            duration_ms,
            mode: HistoryMode::Receive,
            peer_addr,
            peer_username: None,
            receiver_path: None,
            description: None,
            status: HistoryStatus::Failed,
            error: Some(format!("{:#}", e)),
            intended_count: expected_items.unwrap_or(0),
            intended_bytes: 0,
            intended_items: None,
            actual_count: 0,
            actual_bytes: 0,
            actual_items: None,
        };
        if let Err(err) = append_record(&record).await {
            error!("Failed to append failed history record: {:#}", err);
        } else {
            info!("Successfully appended failed receive history record.");
            trace!("Appended failed record details: {:?}", record);
        }
    }

    result
}
