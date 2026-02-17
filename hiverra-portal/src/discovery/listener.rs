use crate::discovery::protocol::PortalBeacon;
use anyhow::Result;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;

pub async fn find_receiver(target_username: &str) -> Result<(String, String, u16)> {
    // create the low-level socket for OS port-sharing
    let raw_socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;

    raw_socket.set_reuse_address(true)?;
    #[cfg(not(windows))]
    raw_socket.set_reuse_port(true)?;

    let address: SocketAddr = "0.0.0.0:5005".parse()?;
    raw_socket.set_nonblocking(true)?;
    raw_socket.bind(&address.into())?;

    // convert to Tokio's async UdpSocket
    let std_socket: std::net::UdpSocket = raw_socket.into();
    let socket = UdpSocket::from_std(std_socket)?;

    // join the multicast group
    let multicast_addr = Ipv4Addr::new(224, 0, 0, 123);
    socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    let mut buf = [0u8; 1024];

    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf).await?;

        if let Ok(beacon) = serde_json::from_slice::<PortalBeacon>(&buf[..len]) {
            // check if this is the person we are looking for
            if beacon.protocol == "portal" && beacon.username == target_username {
                println!("Portal: Found {} at {}!", beacon.username, remote_addr.ip());

                // return (IP, Node_ID, Port)
                return Ok((remote_addr.ip().to_string(), beacon.node_id, beacon.port));
            }
        }
    }
}
