use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalTransferManifest {
    pub total_files: u32,
    pub total_directories: u32,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectoryMetadata {
    pub dirname: String,
    pub total_size: u64,
    pub files: Vec<FileMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ItemKind {
    File,
    Directory,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum TransferItem {
    File(FileMetadata),
    Directory(DirectoryMetadata),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferHeader {
    pub kind: ItemKind,
}
