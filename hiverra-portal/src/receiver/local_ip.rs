use {
 
    network_interface::{NetworkInterface, NetworkInterfaceConfig},
    std::net::IpAddr
    
};

/// Searches for the first available IPv4 address on common Wi-Fi interface names

pub async fn get_local_ip() -> Option<String> {
    // Retrieve all network interfaces
    let interfaces = NetworkInterface::show().ok()?;

    for interface in interfaces {
        let name = interface.name.to_lowercase();

        // Comprehensive check
        if name.contains("wlan")
            || name.contains("wlp")
            || name.contains("wi-fi")
            || name.starts_with("en")
        {
            for addr in interface.addr {
                // Filter for IPv4 and ignore loopback with check
                if let IpAddr::V4(ipv4) = addr.ip() {
                    if !ipv4.is_loopback() {
                        return Some(ipv4.to_string());
                    }
                }
            }
        }
    }
    None
}
