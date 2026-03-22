use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PortalBeacon {
    pub protocol: String,
    pub node_id: String,
    pub username: String,
    pub port: u16,
}
