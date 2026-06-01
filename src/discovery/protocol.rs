use serde::{Deserialize, Serialize};

pub const DISCOVERY_PORT: u16 = 5005;
pub const MULTICAST_ADDR: &str = "224.0.0.123";
pub const PROTOCOL_NAME: &str = "portal";

#[derive(Serialize, Deserialize, Debug)]
pub struct PortalBeacon {
    pub protocol: String,
    pub node_id: String,
    pub username: String,
    pub port: u16,
}
