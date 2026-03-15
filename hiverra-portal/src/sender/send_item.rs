use {
    crate::metadata::{FileMetadata, PortalMeta, TransferItem},
    crate::sender::create_file_metadata,
    anyhow::{Context, Result},
    async_walkdir::WalkDir,
    bincode::serialize,
    std::path::PathBuf,
    tokio::{fs::File, io::AsyncWrite},
    tokio_stream::StreamExt,
    tokio_tar::{Builder, EntryType, Header},
    tracing::{debug, info, trace, warn},
};

/// Appends a file or directory to the provided tar builder
pub async fn send_item<W>(builder: &mut Builder<W>, path: PathBuf, item: TransferItem) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    match item {
        TransferItem::File(file_meta) => {
            println!(
                "Portal: Preparing to send '{}' ({} bytes)...",
                file_meta.filename, file_meta.file_size
            );

            // Wrap in PortalMeta::Item and send metadata
            debug!("Serializing metadata for file: {}", file_meta.filename);
            let meta_bytes = serialize(&PortalMeta::Item(TransferItem::File(file_meta.clone())))?;
            trace!("Serialized file metadata size: {} bytes", meta_bytes.len());
            append_raw_meta(builder, meta_bytes).await?;

            trace!("Opening file for reading: {:?}", path);
            let mut file = File::open(&path).await?;
            let mut header = Header::new_gnu();
            header.set_path(&file_meta.filename)?;
            header.set_size(file_meta.file_size);
            header.set_mode(0o644);
            header.set_cksum();

            trace!("Appending file '{}' to tar archive", file_meta.filename);
            builder.append(&header, &mut file).await?;

            println!("Portal: File '{}' sent successfully!", file_meta.filename);
            info!(
                "File '{}' transfer initiated and appended to stream.",
                file_meta.filename
            );
        }

        TransferItem::Directory(dir_meta) => {
            // tell user they are sending empty dir if empty
            if dir_meta.total_size == 0 {
                println!(
                    "Portal: Note: Directory '{}' is empty. Sending structure only.",
                    dir_meta.dirname
                );
                warn!(
                    "Directory '{}' is empty; sending structure only.",
                    dir_meta.dirname
                );
            } else {
                println!(
                    "Portal: Preparing to send directory '{}' ({} bytes)...",
                    &dir_meta.dirname, dir_meta.total_size
                );
            }

            // Top-level directory metadata
            debug!("Serializing metadata for directory: {}", dir_meta.dirname);
            let meta_bytes =
                serialize(&PortalMeta::Item(TransferItem::Directory(dir_meta.clone())))?;
            trace!(
                "Serialized directory metadata size: {} bytes",
                meta_bytes.len()
            );
            append_raw_meta(builder, meta_bytes).await?;

            // Append directory entry itself
            trace!(
                "Appending directory node '{}' to tar archive",
                dir_meta.dirname
            );
            let mut dir_header = Header::new_gnu();
            dir_header.set_path(&dir_meta.dirname)?;
            dir_header.set_entry_type(EntryType::Directory);
            dir_header.set_mode(0o755);
            dir_header.set_size(0);
            dir_header.set_cksum();
            builder.append(&dir_header, &[][..]).await?;

            // Stream the contents of the directory
            debug!("Starting WalkDir for directory: {:?}", path);
            let mut entries = WalkDir::new(&path);
            while let Some(entry) = entries.next().await {
                let entry = entry.context("Portal: Failed to read directory entry")?;
                let file_type = entry.file_type().await?;
                let local_path = entry.path();
                let rel_path = local_path.strip_prefix(&path)?;
                let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                let tar_path = format!("{}/{}", dir_meta.dirname, rel_path_str);

                trace!(
                    "Processing entry: {:?} -> tar_path: {}",
                    local_path, tar_path
                );

                if file_type.is_file() {
                    // Nested file metadata
                    debug!("Processing nested file: {}", tar_path);
                    let mut file_meta = create_file_metadata(&local_path).await?;
                    file_meta.filename = tar_path.clone();

                    trace!("Serializing nested file metadata for: {}", tar_path);
                    let meta_bytes = serialize(&PortalMeta::NestedFile(file_meta.clone()))?;
                    trace!("Nested file metadata size: {} bytes", meta_bytes.len());
                    append_raw_meta(builder, meta_bytes).await?;

                    trace!("Opening nested file: {:?}", local_path);
                    let mut file = File::open(&local_path).await?;
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_size(file.metadata().await?.len());
                    header.set_mode(0o644);
                    header.set_cksum();

                    trace!("Appending nested file '{}' to tar archive", tar_path);
                    builder.append(&header, &mut file).await?;

                    info!("Directory file sent successfully: {}", &tar_path);
                } else if file_type.is_dir() {
                    // Subdirectory header
                    debug!("Processing nested directory: {}", tar_path);
                    let sub_dir_meta = FileMetadata {
                        filename: tar_path.clone(),
                        file_size: 0,
                    };

                    trace!("Serializing nested directory metadata for: {}", tar_path);
                    let meta_bytes = serialize(&PortalMeta::NestedFile(sub_dir_meta))?;
                    trace!("Nested directory metadata size: {} bytes", meta_bytes.len());
                    append_raw_meta(builder, meta_bytes).await?;

                    trace!("Appending subdirectory entry to tar: {}", tar_path);
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_entry_type(EntryType::Directory);
                    header.set_mode(0o755);
                    header.set_size(0);
                    header.set_cksum();
                    builder.append(&header, &[][..]).await?;
                }
            }

            println!(
                "Portal: Directory '{}' sent successfully!",
                dir_meta.dirname
            );
            info!("Directory '{}' transfer complete.", dir_meta.dirname);
        }
    }

    Ok(())
}

/// Helper to write the bincode metadata as a hidden virtual file in the tar stream
async fn append_raw_meta<W: AsyncWrite + Unpin + Send>(
    builder: &mut Builder<W>,
    bytes: Vec<u8>,
) -> Result<()> {
    debug!(
        "Appending metadata header (.portal.meta) - size: {} bytes",
        bytes.len()
    );
    trace!("Metadata payload content: {:?}", bytes);
    let mut header = Header::new_gnu();
    header.set_path(".portal.meta")?;
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    builder.append(&header, &bytes[..]).await?;
    Ok(())
}
