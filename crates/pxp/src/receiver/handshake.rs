use {
    crate::{
        discovery::beacon::start_beacon,
        metadata::GlobalTransferManifest,
    },
    crate::error::{PxpError, Result},
    bincode::deserialize,
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
}

/// Accept a connection, run discovery beacon, verify identity, and read manifest.
/// This is the core protocol handshake — no config loading or user-facing output.
pub async fn accept_and_read_manifest(
    port: u16,
    username: String,
) -> Result<HandshakeResult> {
    let node_id = Uuid::new_v4().to_string();
    debug!("Generated session Node ID: {}", node_id);

    let bind_addr = format!("0.0.0.0:{}", port);
    trace!("Listener target address: {}", bind_addr);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| PxpError::BindFailed { port, source: e })?;

    info!("TCP Listener bound to {}", bind_addr);

    // Run beacon and TCP accept concurrently
    let (mut socket, addr) = tokio::select! {
        _ = start_beacon(username, node_id.clone(), port) => {
            error!("Discovery beacon exited unexpectedly");
            return Err(PxpError::BeaconStopped);
        }
        result = listener.accept() => {
            let (conn, addr) = result?;
            trace!("Accepted raw TCP connection from: {:?}", addr);
            (conn, addr)
        }
    };

    info!("Connection accepted from sender: {}", addr);
    let peer_addr = Some(addr.ip().to_string());

    // Send ID to Sender so they can verify who we are
    debug!("Sending Node ID for verification: {}", node_id);
    let id_bytes = node_id.as_bytes();
    let id_len = id_bytes.len() as u32;
    trace!("Node ID length: {} bytes", id_len);

    socket
        .write_all(&id_len.to_be_bytes())
        .await?;
    socket
        .write_all(id_bytes)
        .await?;
    trace!("Verification identity sent to peer.");

    // Read the manifest length
    let mut global_manifest_len_buf = [0u8; 4];
    socket
        .read_exact(&mut global_manifest_len_buf)
        .await?;

    let global_manifest_len = u32::from_be_bytes(global_manifest_len_buf) as usize;
    debug!(
        "Incoming global manifest length: {} bytes",
        global_manifest_len
    );

    // Read the manifest blob
    let mut global_manifest_buf = vec![0u8; global_manifest_len];
    socket
        .read_exact(&mut global_manifest_buf)
        .await?;
    trace!(
        "Read global manifest raw bytes (size: {}). Deserializing...",
        global_manifest_len
    );

    let manifest: GlobalTransferManifest =
        deserialize(&global_manifest_buf)?;

    info!("Global manifest received and deserialized successfully.");
    trace!("Manifest data: {:?}", manifest);

    Ok(HandshakeResult {
        socket,
        peer_addr,
        peer_username: manifest.sender_username.clone(),
        manifest,
    })
}
