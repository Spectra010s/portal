use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalTransferManifest {
    pub total_files: u32,
    pub total_directories: u32,
    pub total_bytes: u64,
    pub description: Option<String>,
    pub sender_username: Option<String>,
    pub compressed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryMetadata {
    pub dirname: String,
    pub total_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransferItem {
    File(FileMetadata),
    Directory(DirectoryMetadata),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PxpMeta {
    Item(TransferItem),
    NestedFile(FileMetadata),
}

/// A single item that was received during a transfer.
#[derive(Debug, Clone)]
pub struct ReceivedItem {
    pub name: String,
    pub bytes: u64,
    pub is_directory: bool,
}

/// Summary of items received during a transfer.
#[derive(Debug, Clone)]
pub struct ReceiveSummary {
    pub items: Vec<ReceivedItem>,
    pub total_bytes: u64,
}
