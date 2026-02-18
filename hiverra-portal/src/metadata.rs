use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferManifest {
    pub files: Vec<FileMetadata>,
    pub total_files: u32,
    // description from sender to receiver
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
}
