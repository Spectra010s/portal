use serde::{Deserialize, Serialize};

// We 'derive' these traits so Serde knows how to convert this
#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
    // so the sender can send a description of some sort
    // Option allows us to have a description or nothing (None)
    pub description: Option<String>,
}
