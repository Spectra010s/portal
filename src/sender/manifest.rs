use {
    crate::metadata::{DirectoryMetadata, FileMetadata, GlobalTransferManifest},
    anyhow::{Context, Result},
    async_walkdir::WalkDir,
    std::path::PathBuf,
    tokio::fs::metadata,
    tokio_stream::StreamExt,
    tracing::{debug, trace},
};

pub async fn create_file_metadata(path: &PathBuf) -> Result<FileMetadata> {
    let attr = metadata(path).await?;
    debug!("Generating metadata for file: {:?}", path);

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file")
        .to_string();

    Ok(FileMetadata {
        filename,
        file_size: attr.len(),
    })
}

pub async fn create_directory_metadata(dir: &PathBuf) -> Result<DirectoryMetadata> {
    debug!("Calculating total size for directory: {:?}", dir);
    let mut total_size = 0u64;
    let mut entries = WalkDir::new(dir);
    // We walk the directory once to get the total size for the Global Manifest
    while let Some(entry) = entries.next().await {
        let entry = entry.context("Portal: Failed to read directory entry for metadata")?;
        let entry_path = entry.path();
        trace!("Scanning path for size calculation: {:?}", entry_path);

        let file_type = entry.file_type().await?;

        if file_type.is_file() {
            if let Ok(meta) = entry.metadata().await {
                trace!("Found file: {:?} ({} bytes)", entry_path, meta.len());
                total_size += meta.len();
            }
        }
    }

    debug!(
        "Directory size calculation complete: {} bytes total for {:?}",
        total_size, dir
    );
    Ok(DirectoryMetadata {
        dirname: dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_dir")
            .to_string(),
        total_size,
    })
}

pub async fn create_global_transfer_manifest(
    files: u32,
    dirs: u32,
    total_bytes: u64,
    desc: Option<String>,
    sender_username: Option<String>,
    compressed: bool,
) -> Result<GlobalTransferManifest> {
    debug!(
        "Global Manifest: {} files, {} dirs, {} bytes, sender_username={:?}, compressed={}",
        files, dirs, total_bytes, sender_username, compressed
    );
    Ok(GlobalTransferManifest {
        total_files: files,
        total_directories: dirs,
        total_bytes,
        description: desc,
        sender_username,
        compressed,
    })
}
