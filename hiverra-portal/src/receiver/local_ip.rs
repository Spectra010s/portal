use {
    network_interface::{NetworkInterface, NetworkInterfaceConfig},
    std::net::IpAddr,
    tracing::{debug, trace, warn},
};

/// Searches for the first available IPv4 address on common Wi-Fi interface names

pub async fn get_local_ip() -> Option<String> {
    // Retrieve all network interfaces
    trace!("Retrieving all network interfaces...");
    let interfaces = NetworkInterface::show().ok()?;
    trace!("Found {} interfaces", interfaces.len());

    for interface in interfaces {
        let name = interface.name.to_lowercase();
        trace!(
            "Checking interface: {} (addr count: {})",
            name,
            interface.addr.len()
        );

        // Comprehensive check
        if name.contains("wlan")
            || name.contains("wlp")
            || name.contains("wi-fi")
            || name.starts_with("en")
        {
            debug!(
                "Interface '{}' matches search criteria, scanning addresses...",
                name
            );
            for addr in interface.addr {
                trace!("Found address: {:?}", addr.ip());
                // Filter for IPv4 and ignore loopback with check
                if let IpAddr::V4(ipv4) = addr.ip() {
                    if !ipv4.is_loopback() {
                        debug!("Selected suitable local IPv4: {}", ipv4);
                        return Some(ipv4.to_string());
                    }
                }
            }
        }
    }
    warn!("No suitable local IPv4 address found on standard interfaces.");
    None
}
