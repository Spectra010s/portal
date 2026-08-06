use {
    crate::discovery::listener::{find_receiver_broadcast, find_receiver_multicast},
    crate::error::{PxpError, Result},
    std::time::Duration,
    tokio::{io::AsyncReadExt, net::TcpStream, time::timeout},
    tracing::{debug, error, info, trace, warn},
};

/// Discover a receiver by username, trying multicast first then broadcast.
/// Returns (ip, node_id, port) on success.
pub async fn discover_receiver(
    target_username: &str,
    fallback_port: u16,
) -> Result<(String, String, u16)> {
    info!("Discovery started for user: {}", target_username);

    let discovery_result = match timeout(
        Duration::from_secs(30),
        find_receiver_multicast(target_username),
    )
    .await
    {
        Ok(result) => result?,
        Err(_) => {
            warn!("Multicast discovery timed out for user: {}", target_username);
            warn!("Trying subnet broadcast discovery for user: {}", target_username);

            match timeout(
                Duration::from_secs(30),
                find_receiver_broadcast(target_username),
            )
            .await
            {
                Ok(result) => result?,
                Err(_) => {
                    warn!("Broadcast discovery timed out for user: {}", target_username);
                    return Err(PxpError::DiscoveryTimeout {
                        message: format!(
                            "Search timed out. Make sure the receiver is active and on the same network.\n\
                             Portal: Try direct address mode:\n\
                             Portal:   portal send --address <receiver-ip> --port {} <file-or-folder>\n\
                             Tip: The receiver shows its listening address when running `portal receive`.",
                            fallback_port
                        )
                    });
                }
            }
        }
    };

    let (ip, id, p) = discovery_result;
    info!("Receiver found at {}:{} (Node ID: {})", ip, p, id);
    Ok((ip, id, p))
}

/// Connect to a receiver at the given address and verify its identity.
/// If `expected_node_id` is Some, the receiver's claimed ID must match.
pub async fn connect_to_receiver(
    target_ip: &str,
    target_port: u16,
    expected_node_id: Option<&str>,
) -> Result<TcpStream> {
    let r_addr = format!("{}:{}", target_ip, target_port);

    let mut stream = TcpStream::connect(&r_addr)
        .await
        .map_err(|e| PxpError::ConnectionFailed { address: r_addr.clone(), source: e })?;
    info!("TCP connection established with {}", r_addr);

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
    if let Some(expected_id) = expected_node_id {
        trace!(
            "Verifying claimed ID against expected beacon ID: {}",
            expected_id
        );
        if claimed_id != expected_id {
            error!(
                "SECURITY ALERT: Claimed ID {} does not match beacon ID {}",
                claimed_id, expected_id
            );
            return Err(PxpError::IdentityMismatch { claimed: claimed_id, expected: expected_id.to_string() });
        }
        info!("Identity verified via node ID match.");
    } else {
        warn!("Direct IP mode used: skipping identity verification.");
    }

    Ok(stream)
}
