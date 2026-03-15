use {
    crate::discovery::protocol::PortalBeacon,
    anyhow::Result,
    std::time::Duration,
    tokio::net::UdpSocket,
    tracing::{debug, info, trace},
};

pub async fn start_beacon(username: String, node_id: String, tcp_port: u16) -> Result<()> {
    info!("Portal: Starting discovery beacon for '{}'", username);

    // bind anywhere
    trace!("Binding discovery UDP socket to 0.0.0.0:0");
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let target_addr = "224.0.0.123:5005";
    trace!("Multicast target address set to: {}", target_addr);

    let beacon = PortalBeacon {
        protocol: "portal".to_string(),
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
        // sends to the multicast address
        trace!("Broadcasting discovery heartbeat...");
        socket.send_to(&msg, target_addr).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
