use crate::discovery::protocol::PortalBeacon;
use anyhow::Result;
use std::time::Duration;
use tokio::net::UdpSocket;

pub async fn start_beacon(username: String, node_id: String, tcp_port: u16) -> Result<()> {
    // bind anywhere
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let target_addr = "224.0.0.123:5005";

    let beacon = PortalBeacon {
        protocol: "portal".to_string(),
        node_id,
        username,
        port: tcp_port,
    };

    let msg = serde_json::to_vec(&beacon)?;

    loop {
        // sends to the multicast address
        socket.send_to(&msg, target_addr).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
