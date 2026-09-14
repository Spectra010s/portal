use {
    crate::discovery::listener::{find_receiver_broadcast, find_receiver_multicast},
    crate::error::{PxpError, Result},
    std::time::Duration,
    tokio::{io::AsyncReadExt, net::TcpStream, time::timeout},
    tracing::{debug, error, info, trace, warn},
};

/// Maximum number of TCP connection attempts before giving up.
const CONNECT_MAX_ATTEMPTS: u32 = 3;

/// Base delay between retry attempts. Doubles on each retry (exponential backoff).
const CONNECT_RETRY_BASE_DELAY: Duration = Duration::from_secs(2);

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
                        ),
                    });
                }
            }
        }
    };

    let (ip, id, p) = discovery_result;
    info!("Receiver found at {}:{} (Node ID: {})", ip, p, id);
    Ok((ip, id, p))
}

/// Connect to a receiver at the given address and verify its identity, retrying on
/// transient failures with exponential backoff.
///
/// Attempts up to [`CONNECT_MAX_ATTEMPTS`] times before returning the last error.
/// Identity mismatches are treated as fatal and are never retried — a mismatched
/// node ID means a different host answered, not a transient network blip.
///
/// If `expected_node_id` is `Some`, the receiver's claimed session ID must match
/// the value observed in the discovery beacon. Pass `None` for direct-IP mode.
pub async fn connect_to_receiver(
    target_ip: &str,
    target_port: u16,
    expected_node_id: Option<&str>,
) -> Result<TcpStream> {
    let mut last_err: Option<PxpError> = None;

    for attempt in 1..=CONNECT_MAX_ATTEMPTS {
        if attempt > 1 {
            // Exponential backoff: 2s, 4s, …
            let delay = CONNECT_RETRY_BASE_DELAY * 2u32.pow(attempt - 2);
            warn!(
                "Connection attempt {}/{} failed. Retrying in {}s...",
                attempt - 1,
                CONNECT_MAX_ATTEMPTS,
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }

        match attempt_connect(target_ip, target_port, expected_node_id).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                // Identity mismatches are fatal — never retry them.
                if matches!(e, PxpError::IdentityMismatch { .. }) {
                    error!("Identity mismatch is fatal; aborting retry loop.");
                    return Err(e);
                }
                debug!("Attempt {}/{} error: {}", attempt, CONNECT_MAX_ATTEMPTS, e);
                last_err = Some(e);
            }
        }
    }

    Err(last_err.expect("retry loop ran at least once"))
}

/// Single connection attempt: open TCP, read the identity proof, verify it.
async fn attempt_connect(
    target_ip: &str,
    target_port: u16,
    expected_node_id: Option<&str>,
) -> Result<TcpStream> {
    let r_addr = format!("{}:{}", target_ip, target_port);

    let mut stream = TcpStream::connect(&r_addr)
        .await
        .map_err(|e| PxpError::ConnectionFailed { address: r_addr.clone(), source: e })?;
    info!("TCP connection established with {}", r_addr);

    // Read the session ID the receiver is asserting over the wire.
    debug!("Reading receiver identity proof...");
    let mut id_len_buf = [0u8; 4];
    stream.read_exact(&mut id_len_buf).await?;
    let id_len = u32::from_be_bytes(id_len_buf) as usize;

    // Guard against a malformed or malicious length prefix before allocating.
    if id_len > 1024 {
        return Err(PxpError::Protocol(format!(
            "Receiver sent an implausibly long session ID ({} bytes); closing connection.",
            id_len
        )));
    }
    trace!("Receiver claimed ID length: {} bytes", id_len);

    let mut id_buf = vec![0u8; id_len];
    stream.read_exact(&mut id_buf).await?;
    let claimed_id = String::from_utf8(id_buf)?;
    trace!("Receiver claimed ID: {}", claimed_id);

    // In discovery mode, compare the TCP-reported ID against the beacon ID.
    // This prevents a race condition where a different host binds the same port
    // between the moment we saw the beacon and the moment we connected.
    if let Some(expected_id) = expected_node_id {
        trace!(
            "Verifying claimed ID against expected beacon ID: {}",
            expected_id
        );
        if claimed_id != expected_id {
            error!(
                "SECURITY ALERT: Claimed ID '{}' does not match beacon ID '{}'",
                claimed_id, expected_id
            );
            return Err(PxpError::IdentityMismatch {
                claimed: claimed_id,
                expected: expected_id.to_string(),
            });
        }
        info!("Identity verified via node ID match.");
    } else {
        warn!("Direct IP mode: skipping identity verification.");
    }

    Ok(stream)
}
