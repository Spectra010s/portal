mod get_dir;
mod local_ip;
mod receive_item;

use {
    crate::{
        config::models::PortalConfig, discovery::beacon::start_beacon,
        metadata::GlobalTransferManifest,
    },
    anyhow::{Context, Result, anyhow},
    async_compression::tokio::bufread::GzipDecoder,
    bincode::deserialize,
    get_dir::get_target_dir,
    local_ip::get_local_ip,
    receive_item::receive_item,
    std::path::PathBuf,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt, BufReader},
        net::TcpListener,
    },
    tokio_tar::Archive,
    tracing::{debug, error, info, warn},
    uuid::Uuid,
};

pub async fn start_receiver(port: Option<u16>, dir: &Option<PathBuf>) -> Result<()> {
    info!("Portal: Initializing receiver systems...");

    let my_ip = get_local_ip()
        .await
        .context("Failed to get IP address, pls try again")?;
    debug!("Local IP detected: {:?}", my_ip);

    // Use the CLI flag directly
    let n_port = if let Some(port) = port {
        debug!("Portal: Overriding config port with CLI port: {}", port);
        port
    } else if let Some(cfg) = PortalConfig::load_or_return().await? {
        //  Use config if it exists and has a value
        if let Some(p) = cfg.network.default_port {
            debug!("Portal: Port not given, using config port: {}", p);
            p
        } else {
            error!("Port missing in both CLI and config");
            return Err(anyhow!("No port provided and config has no port set"));
        }
    } else {
        //  Neither CLI nor config
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
            result.context("Failed to accept connection")?
        }
    };

    info!("Connection accepted from sender: {}", addr);
    println!("Portal: Connection established with {}!", addr);
    println!("Portal: Connected to sender");
    println!("Portal: Waiting for incoming files...");

    // Send ID to Sender so they can verify who we are
    debug!("Sending Node ID for verification: {}", node_id);
    let id_bytes = node_id.as_bytes();
    let id_len = id_bytes.len() as u32;

    socket
        .write_all(&id_len.to_be_bytes())
        .await
        .context("Failed to send verification length")?;
    socket
        .write_all(id_bytes)
        .await
        .context("Failed to send verification ID")?;

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

    // Deserialize the manifest
    let global_manifest: GlobalTransferManifest =
        deserialize(&global_manifest_buf).context("Failed to deserialize global manifest")?;

    info!("Global manifest received and deserialized successfully.");

    let total_directories = &global_manifest.total_directories;
    let total_files = global_manifest.total_files;
    let description = &global_manifest.description;

    let total_items = total_files + total_directories;

    // Print basic info for the user
    println!("Portal: Incoming transfer - {} item(s)", total_items);

    if let Some(desc) = description {
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

    // receive the item: file or directory
    receive_item(&mut archive, &target_dir, total_items).await?;

    debug!("Extraction complete. Recovering stream...");
    let decoder = archive.into_inner().map_err(|_| {
        error!("Failed to recover decoder from archive");
        anyhow!("Failed to recover decoder from archive")
    })?;

    let _socket = decoder.into_inner();

    info!(
        "SUCCESS: Transfer completed. Saved to {}",
        target_dir.display()
    );
    println!(
        "Portal: All item(s) have been received successfully! Saved to '{}'",
        target_dir.display()
    );

    Ok(())
}
