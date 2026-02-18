use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferManifest {
    pub files: Vec<FileMetadata>,
    pub total_size: u64,
    // description from sender to receiver
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
}
