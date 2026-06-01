use {
    crate::discovery::protocol::{DISCOVERY_PORT, MULTICAST_ADDR, PROTOCOL_NAME, PortalBeacon},
    anyhow::Result,
    network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig},
    std::collections::BTreeSet,
    std::time::Duration,
    tokio::net::UdpSocket,
    tracing::{debug, info, trace, warn},
};

pub async fn start_beacon(username: String, node_id: String, tcp_port: u16) -> Result<()> {
    info!("Portal: Starting discovery beacon for '{}'", username);

    // bind anywhere
    trace!("Binding discovery UDP socket to 0.0.0.0:0");
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let multicast_target = format!("{}:{}", MULTICAST_ADDR, DISCOVERY_PORT);
    let broadcast_targets = broadcast_targets();
    trace!("Multicast target address set to: {}", multicast_target);
    debug!("Broadcast target addresses set to: {:?}", broadcast_targets);

    let beacon = PortalBeacon {
        protocol: PROTOCOL_NAME.to_string(),
        node_id,
        username,
        port: tcp_port,
    };

    let msg = serde_json::to_vec(&beacon)?;
    debug!(
        "Discovery payload prepared ({} bytes): {:?}",
        msg.len(),
        beacon
    );

    loop {
        trace!("Sending multicast discovery heartbeat...");
        socket.send_to(&msg, &multicast_target).await?;

        for target_addr in &broadcast_targets {
            trace!("Sending broadcast discovery heartbeat to {}...", target_addr);
            if let Err(err) = socket.send_to(&msg, target_addr).await {
                warn!("Failed to send broadcast discovery heartbeat: {}", err);
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn broadcast_targets() -> Vec<String> {
    let mut targets = BTreeSet::new();

    match NetworkInterface::show() {
        Ok(interfaces) => {
            for interface in interfaces {
                if interface.internal {
                    continue;
                }

                for addr in interface.addr {
                    if let Addr::V4(ifaddr) = addr {
                        if ifaddr.ip.is_loopback() {
                            continue;
                        }

                        if let Some(broadcast) = ifaddr.broadcast {
                            targets.insert(format!("{}:{}", broadcast, DISCOVERY_PORT));
                        }
                    }
                }
            }
        }
        Err(err) => {
            debug!("Could not inspect network interfaces for broadcast targets: {}", err);
        }
    }

    if targets.is_empty() {
        targets.insert(format!("255.255.255.255:{}", DISCOVERY_PORT));
    }

    targets.into_iter().collect()
}
