use {
    crate::{
        config::models::PortalConfig, discovery::beacon::start_beacon,
        history::TransferHistoryRecord, metadata::GlobalTransferManifest,
        receiver::local_ip::get_local_ip,
    },
    anyhow::{Context, Result, anyhow},
    bincode::deserialize,
    std::time::Instant,
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    },
    tracing::{debug, error, info, trace},
    uuid::Uuid,
};

pub struct HandshakeResult {
    pub socket: TcpStream,
    pub peer_addr: Option<String>,
    pub peer_username: Option<String>,
    pub manifest: GlobalTransferManifest,
    pub start_ts_unix: u64,
    pub start_instant: Instant,
}

pub async fn accept_and_read_manifest(port: Option<u16>) -> Result<HandshakeResult> {
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
    let peer_addr = Some(addr.ip().to_string());
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
    let start_ts_unix = TransferHistoryRecord::now_unix();
    let start_instant = Instant::now();
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
    let manifest: GlobalTransferManifest =
        deserialize(&global_manifest_buf).context("Failed to deserialize global manifest")?;

    info!("Global manifest received and deserialized successfully.");
    trace!("Manifest data: {:?}", manifest);

    Ok(HandshakeResult {
        socket,
        peer_addr,
        peer_username: manifest.sender_username.clone(),
        manifest,
        start_ts_unix,
        start_instant,
    })
}
