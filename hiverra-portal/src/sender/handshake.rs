use {
    crate::discovery::listener::find_receiver,
    anyhow::{Context, Result, anyhow},
    inquire::Text,
    std::time::Duration,
    tokio::{
        io::AsyncReadExt,
        net::TcpStream,
        time::timeout,
    },
    tracing::{debug, error, info, trace, warn},
};

pub async fn connect_and_verify(
    addr: &Option<String>,
    port: &u16,
    to: &Option<String>,
) -> Result<(TcpStream, String, Option<String>, Option<String>)> {
    let mut peer_username: Option<String> = None;

    // Username discovery connection logic
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
            find_receiver(&target_username),
        )
        .await
        .context("Portal: Search timed out. Make sure the receiver is active and on the same network.")??;

        let (ip, id, p) = discovery_result;
        info!("Receiver found at {}:{} (Node ID: {})", ip, p, id);
        (ip, Some(id), p)
    };

    let r_addr = format!("{}:{}", target_ip, target_port);
    let peer_addr = Some(target_ip.clone());
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

    Ok((stream, r_addr, peer_addr, peer_username))
}
