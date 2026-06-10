use {
    crate::discovery::protocol::{DISCOVERY_PORT, MULTICAST_ADDR, PROTOCOL_NAME, PortalBeacon},
    anyhow::Result,
    socket2::{Domain, Protocol, Socket, Type},
    std::net::{Ipv4Addr, SocketAddr},
    tokio::net::UdpSocket,
    tracing::{debug, info, trace},
};

#[derive(Debug, Clone, Copy)]
pub enum DiscoveryMode {
    Multicast,
    Broadcast,
}

pub async fn find_receiver_multicast(target_username: &str) -> Result<(String, String, u16)> {
    find_receiver(target_username, DiscoveryMode::Multicast).await
}

pub async fn find_receiver_broadcast(target_username: &str) -> Result<(String, String, u16)> {
    find_receiver(target_username, DiscoveryMode::Broadcast).await
}

async fn find_receiver(
    target_username: &str,
    mode: DiscoveryMode,
) -> Result<(String, String, u16)> {
    // create the low-level socket for OS port-sharing
    trace!(
        "Creating raw UDP socket for {:?} discovery (port sharing enabled)",
        mode
    );
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    trace!("Setting SO_REUSEADDR on discovery socket");
    raw_socket.set_reuse_address(true)?;
    #[cfg(not(windows))]
    {
        trace!("Setting SO_REUSEPORT on discovery socket");
        raw_socket.set_reuse_port(true)?;
    }

    let address: SocketAddr = format!("0.0.0.0:{}", DISCOVERY_PORT).parse()?;
    raw_socket.set_nonblocking(true)?;
    trace!("Binding discovery socket to {}", address);
    raw_socket.bind(&address.into())?;

    // convert to Tokio's async UdpSocket
    let std_socket: std::net::UdpSocket = raw_socket.into();
    let socket = UdpSocket::from_std(std_socket)?;

    if let DiscoveryMode::Multicast = mode {
        let multicast_addr: Ipv4Addr = MULTICAST_ADDR.parse()?;
        trace!("Joining multicast group: {}", multicast_addr);
        socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;
    }

    let mut buf = [0u8; 1024];

    trace!("Entering {:?} discovery loop, waiting for beacon...", mode);
    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf).await?;
        trace!(
            "Received packet on port {} (size: {} bytes, from: {})",
            DISCOVERY_PORT, len, remote_addr
        );

        if let Ok(beacon) = serde_json::from_slice::<PortalBeacon>(&buf[..len]) {
            trace!(
                "Deserialized beacon: protocol='{}', user='{}'",
                beacon.protocol, beacon.username
            );
            // check if this is the person we are looking for
            if beacon.protocol == PROTOCOL_NAME {
                if beacon.username == target_username {
                    info!(
                        "Portal: Found receiver '{}' at {} via {:?} discovery!",
                        beacon.username,
                        remote_addr.ip(),
                        mode
                    );
                    debug!(
                        "Beacon match found: IP={}, ID={}, Port={}, Mode={:?}",
                        remote_addr.ip(),
                        beacon.node_id,
                        beacon.port,
                        mode
                    );

                    // return (IP, Node_ID, Port)
                    return Ok((remote_addr.ip().to_string(), beacon.node_id, beacon.port));
                } else {
                    debug!(
                        "Beacon username mismatch: expected '{}', got '{}'",
                        target_username, beacon.username
                    );
                }
            } else {
                trace!("Received non-portal beacon or version mismatch.");
            }
        } else {
            trace!("Failed to deserialize incoming UDP packet as PortalBeacon.");
        }
    }
}
