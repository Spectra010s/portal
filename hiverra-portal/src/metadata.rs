use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferManifest {
    pub files: Vec<FileMetadata>,
    pub total_size: u64,    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
    // description from sender
    pub description: Option<String>,
}
